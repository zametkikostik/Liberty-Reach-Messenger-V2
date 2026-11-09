import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../services/vault_service.dart';
import '../services/rust_bridge.dart';
import '../services/chat_repository.dart';

class PeerInfo {
  final String peerId;
  final String bundleJson;
  /// OUR initiator ephemeral (hex) for this peer — attached to outgoing
  /// messages so the responder can bootstrap. '' when we are the responder.
  final String myEphHex;
  PeerInfo(this.peerId, this.bundleJson, this.myEphHex);
}

class PeerRegistry {
  static final instance = PeerRegistry._();
  PeerRegistry._();
  final peers = <String, PeerInfo>{};
  List<PeerInfo> get all => peers.values.toList();
  void add(String peerId, String bundleJson, String myEphHex) {
    peers[peerId] = PeerInfo(peerId, bundleJson, myEphHex);
  }
}

class QrExchangeScreen extends StatefulWidget {
  const QrExchangeScreen({super.key});
  @override
  State<QrExchangeScreen> createState() => _QrExchangeScreenState();
}

class _QrExchangeScreenState extends State<QrExchangeScreen> {
  final _paste = TextEditingController();
  String _myPayload = '';

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final vault = VaultService.instance;
    try {
      await vault.init();
    } on StateError catch (e) {
      if (mounted) setState(() => _myPayload = e.message);
      return;
    }
    final peerId = await vault.peerId();
    final bundle = await vault.preKeyBundle();
    if (mounted) {
      setState(() => _myPayload = jsonEncode({'peerId': peerId, 'bundle': jsonDecode(bundle)}));
    }
  }

  Future<void> _accept() async {
    try {
      final raw = _paste.text.trim();
      if (raw.isEmpty) throw const FormatException('empty payload');
      final map = jsonDecode(raw) as Map<String, dynamic>;
      final peerId = map['peerId'] as String? ?? '';
      final bundle = map['bundle'];
      if (peerId.isEmpty || bundle == null) {
        throw const FormatException('payload must contain peerId and bundle');
      }
      final bundleJson = jsonEncode(bundle);
      // We scanned THEIR payload → we are the initiator.
      final myEph = await RustBridge.instance.sessionStartInitiator(peerId, bundleJson);
      if (myEph.isEmpty) throw StateError('session start failed (invalid bundle?)');
      PeerRegistry.instance.add(peerId, bundleJson, myEph);
      await ChatRepository.instance.saveContacts();
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Session started')));
        Navigator.pop(context);
      }
    } catch (e) {
      if (mounted) ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text('Error: $e')));
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(backgroundColor: const Color(0xFF161B22), title: const Text('QR / Identity')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          const Text('My identity payload', style: TextStyle(color: Colors.white70)),
          SelectableText(_myPayload, style: const TextStyle(color: Colors.white, fontSize: 12)),
          TextButton(onPressed: () => Clipboard.setData(ClipboardData(text: _myPayload)), child: const Text('Copy')),
          const SizedBox(height: 24),
          const Text('Paste peer payload', style: TextStyle(color: Colors.white70)),
          TextField(controller: _paste, maxLines: 4, style: const TextStyle(color: Colors.white, fontSize: 12),
            decoration: const InputDecoration(filled: true, fillColor: Color(0xFF161B22))),
          const SizedBox(height: 12),
          ElevatedButton(
            onPressed: _accept,
            style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF238636)),
            child: const Text('Start session', style: TextStyle(color: Colors.white)),
          ),
        ],
      ),
    );
  }
}
