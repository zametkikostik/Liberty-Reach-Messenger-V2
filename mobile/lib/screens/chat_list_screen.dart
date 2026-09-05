import 'package:flutter/material.dart';
import '../services/vault_service.dart';
import '../services/chat_repository.dart';
import 'chat_screen.dart';
import 'security_screen.dart';
import 'qr_exchange_screen.dart';
import 'groups_screen.dart';
import 'import_screen.dart';
import 'profile_screen.dart';
import 'federation_screen.dart';

class ChatListScreen extends StatefulWidget {
  const ChatListScreen({super.key});
  @override
  State<ChatListScreen> createState() => _ChatListScreenState();
}

class _ChatListScreenState extends State<ChatListScreen> {
  final _vault = VaultService();
  final _repo = ChatRepository.instance;
  VaultMode? _mode;

  @override
  void initState() {
    super.initState();
    _boot();
  }

  Future<void> _boot() async {
    await _repo.init();
    final mode = await _vault.currentMode();
    if (mounted) setState(() => _mode = mode);
  }

  Future<void> _lock() async {
    await _vault.lock();
    if (mounted) Navigator.of(context).pushNamedAndRemoveUntil('/', (r) => false);
  }

  void _openChat(String name, String id) {
    Navigator.of(context).push(MaterialPageRoute(
      builder: (_) => ChatScreen(peerName: name, peerId: id, isRealMode: _mode == VaultMode.real),
    )).then((_) => setState(() {}));
  }

  @override
  Widget build(BuildContext context) {
    final isReal = _mode == VaultMode.real;
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(
        backgroundColor: const Color(0xFF161B22),
        title: const Text('Liberty'),
        actions: [
          if (isReal) IconButton(icon: const Icon(Icons.hub), onPressed: () => Navigator.of(context).push(MaterialPageRoute(builder: (_) => const FederationScreen()))),
          if (isReal) IconButton(icon: const Icon(Icons.person), onPressed: () => Navigator.of(context).push(MaterialPageRoute(builder: (_) => const ProfileScreen()))),
          if (isReal) IconButton(icon: const Icon(Icons.group), onPressed: () => Navigator.of(context).push(MaterialPageRoute(builder: (_) => const GroupsScreen()))),
          if (isReal) IconButton(icon: const Icon(Icons.qr_code), onPressed: () => Navigator.of(context).push(MaterialPageRoute(builder: (_) => const QrExchangeScreen())).then((_) async { await _repo.saveContacts(); setState(() {}); })),
          IconButton(icon: const Icon(Icons.security), onPressed: () => Navigator.of(context).push(MaterialPageRoute(builder: (_) => const SecurityScreen()))),
          IconButton(icon: const Icon(Icons.lock_outline), onPressed: _lock),
        ],
      ),
      body: Center(
        child: Text(
          isReal ? 'Open QR to add contacts' : 'No conversations yet',
          style: TextStyle(color: Colors.white.withOpacity(0.5)),
        ),
      ),
      floatingActionButton: isReal
          ? FloatingActionButton(
              onPressed: () => Navigator.of(context).push(MaterialPageRoute(builder: (_) => const QrExchangeScreen())),
              backgroundColor: const Color(0xFF238636),
              child: const Icon(Icons.person_add, color: Colors.white),
            )
          : null,
    );
  }
}
