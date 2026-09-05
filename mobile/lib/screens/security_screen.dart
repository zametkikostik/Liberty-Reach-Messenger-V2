import 'package:flutter/material.dart';
import '../services/platform_security.dart';
import '../services/push_service.dart';
import '../services/network_config.dart';
import '../services/auth_service.dart';

class SecurityScreen extends StatefulWidget {
  const SecurityScreen({super.key});
  @override
  State<SecurityScreen> createState() => _SecurityScreenState();
}

class _SecurityScreenState extends State<SecurityScreen> {
  String _keystore = '...';
  final _workerCtrl = TextEditingController();
  bool _workerOk = false;
  bool _swarm = true;

  @override
  void initState() {
    super.initState();
    _load();
  }

  Future<void> _load() async {
    final ks = await PlatformSecurity.keystoreStatus();
    await PushService.instance.loadSavedWorkerUrl();
    final swarm = await NetworkConfig.instance.swarmEnabled();
    if (mounted) setState(() {
      _keystore = ks;
      _workerCtrl.text = PushService.instance.baseUrl;
      _workerOk = PushService.instance.isConfigured;
      _swarm = swarm;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(backgroundColor: const Color(0xFF161B22), title: const Text('Security')),
      body: ListView(
        children: [
          ListTile(title: const Text('Keystore', style: TextStyle(color: Colors.white)), subtitle: Text(_keystore, style: const TextStyle(color: Colors.white54))),
          ListTile(title: const Text('E2EE', style: TextStyle(color: Colors.white)), subtitle: const Text('X3DH + Double Ratchet + AES/ChaCha', style: TextStyle(color: Colors.white54))),
          SwitchListTile(
            title: const Text('libp2p Swarm', style: TextStyle(color: Colors.white)),
            value: _swarm,
            onChanged: (v) async { await NetworkConfig.instance.setSwarmEnabled(v); setState(() => _swarm = v); },
          ),
          Padding(
            padding: const EdgeInsets.all(16),
            child: TextField(
              controller: _workerCtrl,
              style: const TextStyle(color: Colors.white, fontSize: 13),
              decoration: InputDecoration(
                labelText: 'Cloudflare Worker URL',
                labelStyle: const TextStyle(color: Colors.white54),
                filled: true,
                fillColor: const Color(0xFF161B22),
                suffixIcon: IconButton(
                  icon: const Icon(Icons.save, color: Color(0xFF58A6FF)),
                  onPressed: () async {
                    await PushService.instance.setWorkerUrl(_workerCtrl.text.trim());
                    setState(() => _workerOk = PushService.instance.isConfigured);
                  },
                ),
              ),
            ),
          ),
          ListTile(
            title: Text(_workerOk ? 'Wake relay active' : 'Wake relay off', style: const TextStyle(color: Colors.white)),
            subtitle: Text(PushService.instance.deviceId ?? '', style: const TextStyle(color: Colors.white54, fontSize: 11)),
          ),
        ],
      ),
    );
  }
}
