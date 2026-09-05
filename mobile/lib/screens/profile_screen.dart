import 'package:flutter/material.dart';
import '../services/profile_service.dart';
import '../services/vault_service.dart';

class ProfileScreen extends StatefulWidget {
  const ProfileScreen({super.key});
  @override
  State<ProfileScreen> createState() => _ProfileScreenState();
}

class _ProfileScreenState extends State<ProfileScreen> {
  final _name = TextEditingController();
  String _peerId = '';

  @override
  void initState() {
    super.initState();
    ProfileService.instance.load().then((p) {
      _name.text = p.displayName;
      VaultService().peerId().then((id) { if (mounted) setState(() => _peerId = id); });
      setState(() {});
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(
        backgroundColor: const Color(0xFF161B22),
        title: const Text('Profile'),
        actions: [
          TextButton(
            onPressed: () async {
              await ProfileService.instance.save(UserProfile(displayName: _name.text.trim().isEmpty ? 'Liberty User' : _name.text.trim()));
              if (mounted) Navigator.pop(context);
            },
            child: const Text('Save', style: TextStyle(color: Color(0xFF58A6FF))),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          children: [
            TextField(controller: _name, style: const TextStyle(color: Colors.white), decoration: const InputDecoration(labelText: 'Display name', labelStyle: TextStyle(color: Colors.white54))),
            const SizedBox(height: 16),
            SelectableText(_peerId, style: const TextStyle(color: Colors.white70)),
          ],
        ),
      ),
    );
  }
}
