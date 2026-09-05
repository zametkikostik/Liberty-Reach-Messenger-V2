import 'package:flutter/material.dart';
import '../services/group_service.dart';
import '../services/vault_service.dart';

class GroupsScreen extends StatefulWidget {
  const GroupsScreen({super.key});

  @override
  State<GroupsScreen> createState() => _GroupsScreenState();
}

class _GroupsScreenState extends State<GroupsScreen> {
  final _nameController = TextEditingController();
  final _svc = GroupService.instance;
  bool _loading = true;

  @override
  void initState() {
    super.initState();
    _svc.load().then((_) {
      if (mounted) setState(() => _loading = false);
    });
  }

  Future<void> _create() async {
    final name = _nameController.text.trim();
    if (name.isEmpty) return;
    final peer = await VaultService().peerId();
    await _svc.create(name, adminPeer: peer);
    _nameController.clear();
    setState(() {});
    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(content: Text('Group "$name" created')),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final groups = _svc.groups;
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(
        backgroundColor: const Color(0xFF161B22),
        title: const Text('Groups'),
      ),
      body: _loading
          ? const Center(child: CircularProgressIndicator())
          : Column(
              children: [
                Padding(
                  padding: const EdgeInsets.all(16),
                  child: Row(
                    children: [
                      Expanded(
                        child: TextField(
                          controller: _nameController,
                          style: const TextStyle(color: Colors.white),
                          decoration: InputDecoration(
                            hintText: 'Group name',
                            hintStyle: TextStyle(color: Colors.white.withOpacity(0.3)),
                            filled: true,
                            fillColor: const Color(0xFF161B22),
                            border: OutlineInputBorder(
                              borderRadius: BorderRadius.circular(12),
                              borderSide: BorderSide.none,
                            ),
                          ),
                          onSubmitted: (_) => _create(),
                        ),
                      ),
                      const SizedBox(width: 8),
                      IconButton(
                        onPressed: _create,
                        icon: const Icon(Icons.add_circle, color: Color(0xFF238636), size: 36),
                      ),
                    ],
                  ),
                ),
                Expanded(
                  child: groups.isEmpty
                      ? Center(
                          child: Text('No groups yet',
                              style: TextStyle(color: Colors.white.withOpacity(0.4))),
                        )
                      : ListView.builder(
                          itemCount: groups.length,
                          itemBuilder: (_, i) {
                            final g = groups[i];
                            return ListTile(
                              leading: const CircleAvatar(
                                backgroundColor: Color(0xFF1A237E),
                                child: Icon(Icons.group, color: Colors.white),
                              ),
                              title: Text(g.name, style: const TextStyle(color: Colors.white)),
                              subtitle: Text(
                                '${g.members} member(s) · E2EE · ${g.id}',
                                style: TextStyle(color: Colors.white.withOpacity(0.5), fontSize: 12),
                              ),
                            );
                          },
                        ),
                ),
              ],
            ),
    );
  }
}
