import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'push_service.dart';
import 'network_config.dart';

enum FedMode { localMesh, federated, private }

class FedPeer {
  final String name;
  final List<String> multiaddrs;
  final String? wakeUrl;
  final String? privacyUrl;
  final int priority;

  FedPeer({required this.name, this.multiaddrs = const [], this.wakeUrl, this.privacyUrl, this.priority = 100});

  Map<String, dynamic> toJson() => {
        'name': name, 'multiaddrs': multiaddrs, 'wake_url': wakeUrl,
        'privacy_url': privacyUrl, 'priority': priority,
      };

  factory FedPeer.fromJson(Map<String, dynamic> j) => FedPeer(
        name: j['name'] as String? ?? 'peer',
        multiaddrs: (j['multiaddrs'] as List?)?.cast<String>() ?? [],
        wakeUrl: j['wake_url'] as String? ?? j['wakeUrl'] as String?,
        privacyUrl: j['privacy_url'] as String? ?? j['privacyUrl'] as String?,
        priority: j['priority'] as int? ?? 100,
      );
}

class FederationService {
  static final instance = FederationService._();
  FederationService._();

  final _storage = const FlutterSecureStorage();
  FedMode mode = FedMode.federated;
  final List<FedPeer> peers = [];
  bool openDiscovery = true;

  Future<void> load() async {
    final raw = await _storage.read(key: 'federation_v1');
    if (raw == null) return;
    try {
      final j = jsonDecode(raw) as Map<String, dynamic>;
      mode = FedMode.values[(j['mode'] as int?)?.clamp(0, 2) ?? 1];
      openDiscovery = j['open_discovery'] as bool? ?? true;
      peers..clear()..addAll(((j['peers'] as List?) ?? []).map((e) => FedPeer.fromJson(e as Map<String, dynamic>)));
    } catch (_) {}
  }

  Future<void> save() async {
    await _storage.write(
      key: 'federation_v1',
      value: jsonEncode({
        'mode': mode.index,
        'open_discovery': openDiscovery,
        'peers': peers.map((p) => p.toJson()).toList(),
      }),
    );
    final addrs = peers.expand((p) => p.multiaddrs).where((a) => a.isNotEmpty).join(', ');
    await NetworkConfig.instance.setBootstrapPeers(addrs);
    await NetworkConfig.instance.setSwarmEnabled(mode != FedMode.private || peers.isNotEmpty);
    final wake = peers.map((p) => p.wakeUrl).whereType<String>().where((u) => u.isNotEmpty);
    if (wake.isNotEmpty) await PushService.instance.setWorkerUrl(wake.first);
  }

  Future<void> addPeer(FedPeer peer) async {
    final i = peers.indexWhere((p) => p.name == peer.name);
    if (i >= 0) peers[i] = peer; else peers.add(peer);
    await save();
  }

  Future<void> removePeer(String name) async {
    peers.removeWhere((p) => p.name == name);
    await save();
  }

  String exportJson() => jsonEncode({
        'mode': mode.index,
        'open_discovery': openDiscovery,
        'peers': peers.map((p) => p.toJson()).toList(),
      });

  Future<void> importJson(String raw) async {
    final j = jsonDecode(raw) as Map<String, dynamic>;
    mode = FedMode.values[(j['mode'] as int?)?.clamp(0, 2) ?? 1];
    openDiscovery = j['open_discovery'] as bool? ?? true;
    peers..clear()..addAll(((j['peers'] as List?) ?? []).map((e) => FedPeer.fromJson(e as Map<String, dynamic>)));
    await save();
  }

  String modeLabel() {
    switch (mode) {
      case FedMode.localMesh: return 'Local mesh (mDNS LAN)';
      case FedMode.federated: return 'Federated (DHT + bootstrap)';
      case FedMode.private: return 'Private (explicit peers only)';
    }
  }
}
