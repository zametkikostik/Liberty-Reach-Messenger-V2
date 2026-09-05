import 'dart:async';
import 'dart:convert';
import 'package:flutter/foundation.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:http/http.dart' as http;
import 'package:crypto/crypto.dart';
import 'notification_service.dart';
import 'chat_repository.dart';

/// Cloudflare Worker wake relay (NO Firebase).
class PushService {
  static final instance = PushService._();
  PushService._();

  final _storage = const FlutterSecureStorage();
  String? _deviceId;
  String? _hmacSecret;
  String _baseUrl = '';
  Timer? _pollTimer;
  bool _ready = false;

  String? get deviceId => _deviceId;
  String get baseUrl => _baseUrl;
  bool get isConfigured => _baseUrl.isNotEmpty;

  Future<void> init({String workerUrl = '', String hmacSecret = ''}) async {
    if (_ready) return;
    _baseUrl = workerUrl.replaceAll(RegExp(r'/$'), '');
    _hmacSecret = hmacSecret.isEmpty ? null : hmacSecret;
    _deviceId = await _storage.read(key: 'wake_device_id');
    if (_deviceId == null) {
      _deviceId = 'dev-${DateTime.now().millisecondsSinceEpoch}';
      await _storage.write(key: 'wake_device_id', value: _deviceId);
    }
    if (_baseUrl.isNotEmpty) {
      await register();
      _pollTimer = Timer.periodic(const Duration(seconds: 25), (_) => poll());
    }
    ChatRepository.instance.inbound.listen((msg) {
      if (!msg.isMe) {
        NotificationService.instance.show(
          title: msg.peerId.length > 12 ? '${msg.peerId.substring(0, 12)}…' : msg.peerId,
          body: msg.mediaType != null ? '📎 ${msg.mediaType}' : msg.text,
        );
      }
    });
    _ready = true;
  }

  Future<void> setWorkerUrl(String url) async {
    _baseUrl = url.replaceAll(RegExp(r'/$'), '');
    await _storage.write(key: 'wake_worker_url', value: _baseUrl);
    if (_baseUrl.isNotEmpty) {
      await register();
      _pollTimer?.cancel();
      _pollTimer = Timer.periodic(const Duration(seconds: 25), (_) => poll());
    }
  }

  Future<void> loadSavedWorkerUrl() async {
    final u = await _storage.read(key: 'wake_worker_url');
    if (u != null && u.isNotEmpty) await setWorkerUrl(u);
  }

  Map<String, String> _headers(String body) {
    final h = <String, String>{'content-type': 'application/json'};
    if (_hmacSecret != null && _hmacSecret!.isNotEmpty) {
      final mac = Hmac(sha256, utf8.encode(_hmacSecret!));
      h['x-liberty-sig'] = mac.convert(utf8.encode(body)).toString();
    }
    return h;
  }

  Future<void> register({String? route}) async {
    if (_baseUrl.isEmpty || _deviceId == null) return;
    final body = jsonEncode({'device_id': _deviceId, 'route': route ?? _deviceId});
    try {
      final res = await http.post(Uri.parse('$_baseUrl/v1/register'), headers: _headers(body), body: body);
      debugPrint('wake register ${res.statusCode}');
    } catch (e) {
      debugPrint('wake register failed: $e');
    }
  }

  Future<void> wakePeer(String peerRoute, {String? messageId}) async {
    if (_baseUrl.isEmpty) return;
    final body = jsonEncode({
      'target': peerRoute,
      'mid': messageId ?? DateTime.now().millisecondsSinceEpoch.toString(),
    });
    try {
      await http.post(Uri.parse('$_baseUrl/v1/wake'), headers: _headers(body), body: body);
    } catch (e) {
      debugPrint('wake peer failed: $e');
    }
  }

  Future<void> savePeerRoute(String peerId, String route) async {
    final raw = await _storage.read(key: 'peer_wake_routes') ?? '{}';
    final map = jsonDecode(raw) as Map<String, dynamic>;
    map[peerId] = route;
    await _storage.write(key: 'peer_wake_routes', value: jsonEncode(map));
  }

  Future<String?> peerRoute(String peerId) async {
    final raw = await _storage.read(key: 'peer_wake_routes') ?? '{}';
    return (jsonDecode(raw) as Map<String, dynamic>)[peerId] as String?;
  }

  Future<void> poll() async {
    if (_baseUrl.isEmpty || _deviceId == null) return;
    final body = jsonEncode({'device_id': _deviceId});
    try {
      final res = await http.post(Uri.parse('$_baseUrl/v1/poll'), headers: _headers(body), body: body);
      if (res.statusCode != 200) return;
      final wakes = (jsonDecode(res.body) as Map)['wakes'] as List? ?? [];
      for (final w in wakes) {
        await NotificationService.instance.show(title: 'Liberty', body: 'New encrypted message');
      }
    } catch (e) {
      debugPrint('wake poll: $e');
    }
  }

  void dispose() => _pollTimer?.cancel();
}
