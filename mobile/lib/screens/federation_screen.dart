import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../services/federation_service.dart';

class FederationScreen extends StatefulWidget {
  const FederationScreen({super.key});
  @override
  State<FederationScreen> createState() => _FederationScreenState();
}

class _FederationScreenState extends State<FederationScreen> {
  final _fed = FederationService.instance;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _fed.load().then((_) { if (mounted) setState(() => _loading = false); });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(
        backgroundColor: const Color(0xFF161B22),
        title: const Text('Federation & Mesh'),
        actions: [
          IconButton(
            icon: const Icon(Icons.copy),
            onPressed: () {
              Clipboard.setData(ClipboardData(text: _fed.exportJson()));
              ScaffoldMessenger.of(context).showSnackBar(const SnackBar(content: Text('Config copied')));
            },
          ),
        ],
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : ListView(
              padding: const EdgeInsets.all(16),
              children: [
                Text(_fed.modeLabel(), style: const TextStyle(color: Colors.white70)),
                const SizedBox(height: 12),
                ...FedMode.values.map((m) => RadioListTile<FedMode>(
                      value: m,
                      groupValue: _fed.mode,
                      title: Text(m.name, style: const TextStyle(color: Colors.white)),
                      onChanged: (v) async {
                        if (v == null) return;
                        _fed.mode = v;
                        await _fed.save();
                        setState(() {});
                      },
                    )),
                ..._fed.peers.map((p) => ListTile(
                      title: Text(p.name, style: const TextStyle(color: Colors.white)),
                      subtitle: Text('${p.multiaddrs.length} addrs', style: const TextStyle(color: Colors.white54)),
                    )),
              ],
            ),
    );
  }
}
