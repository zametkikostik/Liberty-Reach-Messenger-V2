import 'dart:convert';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class GroupInfo {
  final String id;
  final String name;
  final int members;
  final List<String> memberIds;
  GroupInfo({required this.id, required this.name, required this.members, this.memberIds = const []});
  Map<String, dynamic> toJson() => {'id': id, 'name': name, 'members': members, 'memberIds': memberIds};
  factory GroupInfo.fromJson(Map<String, dynamic> j) => GroupInfo(
    id: j['id'] as String, name: j['name'] as String,
    members: j['members'] as int? ?? 1,
    memberIds: (j['memberIds'] as List?)?.cast<String>() ?? [],
  );
}

class GroupService {
  static final instance = GroupService._();
  GroupService._();
  final _storage = const FlutterSecureStorage();
  final List<GroupInfo> _groups = [];
  List<GroupInfo> get groups => List.unmodifiable(_groups);

  Future<void> load() async {
    final raw = await _storage.read(key: 'groups_v1');
    if (raw == null) return;
    final list = jsonDecode(raw) as List;
    _groups..clear()..addAll(list.map((e) => GroupInfo.fromJson(e as Map<String, dynamic>)));
  }

  Future<void> _save() async {
    await _storage.write(key: 'groups_v1', value: jsonEncode(_groups.map((g) => g.toJson()).toList()));
  }

  Future<GroupInfo> create(String name, {String? adminPeer}) async {
    final g = GroupInfo(
      id: 'grp-${DateTime.now().millisecondsSinceEpoch}',
      name: name, members: 1,
      memberIds: adminPeer != null ? [adminPeer] : [],
    );
    _groups.add(g);
    await _save();
    return g;
  }

  Future<void> addMember(String groupId, String peerId) async {
    final i = _groups.indexWhere((g) => g.id == groupId);
    if (i < 0) return;
    final g = _groups[i];
    if (g.memberIds.contains(peerId)) return;
    final ids = [...g.memberIds, peerId];
    _groups[i] = GroupInfo(id: g.id, name: g.name, members: ids.length, memberIds: ids);
    await _save();
  }
}
