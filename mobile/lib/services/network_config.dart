import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class NetworkConfig {
  static final instance = NetworkConfig._();
  NetworkConfig._();

  final _storage = const FlutterSecureStorage();

  Future<List<String>> bootstrapPeers() async {
    final raw = await _storage.read(key: 'bootstrap_peers') ?? '';
    return raw.split(',').map((s) => s.trim()).where((s) => s.isNotEmpty).toList();
  }

  Future<void> setBootstrapPeers(String csv) async {
    await _storage.write(key: 'bootstrap_peers', value: csv);
  }

  Future<bool> swarmEnabled() async {
    return (await _storage.read(key: 'swarm_enabled')) != '0';
  }

  Future<void> setSwarmEnabled(bool v) async {
    await _storage.write(key: 'swarm_enabled', value: v ? '1' : '0');
  }
}
