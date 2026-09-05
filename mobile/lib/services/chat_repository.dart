import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import '../models/message.dart';
import 'rust_bridge.dart';
import 'media_service.dart';
import 'push_service.dart';
import '../screens/qr_exchange_screen.dart';

class ChatRepository {
  static final instance = ChatRepository._();
  ChatRepository._();

  final _storage = const FlutterSecureStorage();
  final _bridge = RustBridge.instance;
  final _messages = <String, List<ChatMessage>>{};
  final _controllers = <String, StreamController<List<ChatMessage>>>{};
  final _globalIn = StreamController<ChatMessage>.broadcast();
  bool _started = false;

  Stream<ChatMessage> get inbound => _globalIn.stream;

  Stream<List<ChatMessage>> watchChat(String peerId) {
    _controllers.putIfAbsent(peerId, () => StreamController<List<ChatMessage>>.broadcast());
    Future.microtask(() => _controllers[peerId]?.add(List.unmodifiable(_messages[peerId] ?? [])));
    return _controllers[peerId]!.stream;
  }

  Future<void> init() async {
    if (_started) return;
    _started = true;
    await _loadAll();
    await _loadContacts();
  }

  Future<void> _loadAll() async {
    try {
      final raw = await _storage.read(key: 'chat_history_v1');
      if (raw == null) return;
      final map = jsonDecode(raw) as Map<String, dynamic>;
      map.forEach((peerId, list) {
        _messages[peerId] = (list as List).map((e) => ChatMessage.fromJson(e as Map<String, dynamic>)).toList();
      });
    } catch (e) {
      debugPrint('load history: $e');
    }
  }

  Future<void> _saveAll() async {
    final map = <String, dynamic>{};
    _messages.forEach((k, v) => map[k] = v.map((m) => m.toJson()).toList());
    await _storage.write(key: 'chat_history_v1', value: jsonEncode(map));
  }

  Future<void> _loadContacts() async {
    try {
      final raw = await _storage.read(key: 'contacts_v1');
      if (raw == null) return;
      for (final c in jsonDecode(raw) as List) {
        final m = c as Map<String, dynamic>;
        PeerRegistry.instance.add(m['peerId'] as String, m['bundleJson'] as String? ?? '', m['ephHex'] as String? ?? '');
      }
    } catch (_) {}
  }

  Future<void> saveContacts() async {
    final list = PeerRegistry.instance.all.map((p) => {'peerId': p.peerId, 'bundleJson': p.bundleJson, 'ephHex': p.ephHex}).toList();
    await _storage.write(key: 'contacts_v1', value: jsonEncode(list));
  }

  List<ChatMessage> messagesFor(String peerId) => List.unmodifiable(_messages[peerId] ?? []);

  Future<ChatMessage> sendText({required String peerId, required String text, int ttlSecs = 0, String? replyToId, String? replyToText}) async {
    final id = DateTime.now().millisecondsSinceEpoch.toString();
    var msg = ChatMessage(id: id, peerId: peerId, text: text, isMe: true, time: DateTime.now(), status: MessageStatus.sending, ttlSecs: ttlSecs, replyToId: replyToId, replyToText: replyToText);
    _append(msg);
    final peer = PeerRegistry.instance.peers[peerId];
    final encrypted = await _bridge.sessionEncrypt(peerId, text, ephHex: peer?.ephHex ?? '');
    if (encrypted.isEmpty) {
      msg = msg.copyWith(status: MessageStatus.failed);
      _update(msg);
      return msg;
    }
    msg = ChatMessage(id: id, peerId: peerId, text: text, isMe: true, time: msg.time, status: MessageStatus.sent, ttlSecs: ttlSecs, replyToId: replyToId, replyToText: replyToText, encryptedHint: encrypted.length > 36 ? '${encrypted.substring(0, 36)}…' : encrypted);
    _update(msg);
    Future.delayed(const Duration(milliseconds: 400), () => _update(msg.copyWith(status: MessageStatus.delivered)));
    final route = await PushService.instance.peerRoute(peerId) ?? peerId;
    PushService.instance.wakePeer(route, messageId: id);
    return msg;
  }

  Future<ChatMessage> sendMedia({required String peerId, required PickedMedia media, int ttlSecs = 0}) async {
    final path = await MediaService.instance.saveOutbound(peerId, media);
    final id = DateTime.now().millisecondsSinceEpoch.toString();
    final isImage = media.mime.startsWith('image/');
    final msg = ChatMessage(id: id, peerId: peerId, text: isImage ? '' : media.filename, isMe: true, time: DateTime.now(), status: MessageStatus.sent, ttlSecs: ttlSecs, mediaPath: path, mediaType: isImage ? 'image' : 'file', encryptedHint: 'media:${media.size}b');
    _append(msg);
    Future.delayed(const Duration(milliseconds: 400), () => _update(msg.copyWith(status: MessageStatus.delivered)));
    return msg;
  }

  Future<ChatMessage> sendVoice({required String peerId, required String path, required int durationMs, int ttlSecs = 0}) async {
    final id = DateTime.now().millisecondsSinceEpoch.toString();
    final msg = ChatMessage(id: id, peerId: peerId, text: '', isMe: true, time: DateTime.now(), status: MessageStatus.sent, ttlSecs: ttlSecs, mediaPath: path, mediaType: 'voice', durationMs: durationMs);
    _append(msg);
    return msg;
  }

  Future<ChatMessage> forwardMessage({required String toPeerId, required ChatMessage original}) async {
    return sendText(peerId: toPeerId, text: original.text.isNotEmpty ? original.text : 'Forwarded ${original.mediaType ?? "media"}');
  }

  Future<void> receiveText({required String peerId, required String text}) async {
    final msg = ChatMessage(id: 'in-${DateTime.now().millisecondsSinceEpoch}', peerId: peerId, text: text, isMe: false, time: DateTime.now(), status: MessageStatus.delivered);
    _append(msg);
    _globalIn.add(msg);
  }

  Future<void> markRead(String peerId) async {
    final list = _messages[peerId];
    if (list == null) return;
    var changed = false;
    for (var i = 0; i < list.length; i++) {
      if (!list[i].isMe && list[i].status != MessageStatus.read) {
        list[i] = list[i].copyWith(status: MessageStatus.read);
        changed = true;
      }
    }
    if (changed) { _emit(peerId); await _saveAll(); }
  }

  void _append(ChatMessage msg) {
    _messages.putIfAbsent(msg.peerId, () => []);
    _messages[msg.peerId]!.add(msg);
    _emit(msg.peerId);
    _saveAll();
  }

  void _update(ChatMessage msg) {
    final list = _messages[msg.peerId];
    if (list == null) return;
    final i = list.indexWhere((m) => m.id == msg.id);
    if (i >= 0) { list[i] = msg; _emit(msg.peerId); _saveAll(); }
  }

  void _emit(String peerId) => _controllers[peerId]?.add(List.unmodifiable(_messages[peerId] ?? []));
}
