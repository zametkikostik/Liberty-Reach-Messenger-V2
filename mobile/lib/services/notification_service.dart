import 'package:flutter/foundation.dart';
import 'package:flutter_local_notifications/flutter_local_notifications.dart';
import 'chat_repository.dart';

class NotificationService {
  static final instance = NotificationService._();
  NotificationService._();

  final _plugin = FlutterLocalNotificationsPlugin();
  bool _ready = false;

  Future<void> init() async {
    if (_ready) return;
    const android = AndroidInitializationSettings('@mipmap/ic_launcher');
    const ios = DarwinInitializationSettings(
      requestAlertPermission: true,
      requestBadgePermission: true,
      requestSoundPermission: true,
    );
    try {
      await _plugin.initialize(const InitializationSettings(android: android, iOS: ios));
      await _plugin.resolvePlatformSpecificImplementation<AndroidFlutterLocalNotificationsPlugin>()?.requestNotificationsPermission();
    } catch (e) {
      debugPrint('notifications init: $e');
    }
    ChatRepository.instance.inbound.listen((msg) {
      if (!msg.isMe) {
        show(
          title: msg.peerId.length > 12 ? '${msg.peerId.substring(0, 12)}…' : msg.peerId,
          body: msg.mediaType != null ? '📎 ${msg.mediaType}' : msg.text,
        );
      }
    });
    _ready = true;
  }

  Future<void> show({required String title, required String body}) async {
    const details = NotificationDetails(
      android: AndroidNotificationDetails(
        'liberty_messages', 'Messages',
        channelDescription: 'Incoming Liberty messages',
        importance: Importance.high,
        priority: Priority.high,
      ),
      iOS: DarwinNotificationDetails(),
    );
    try {
      await _plugin.show(DateTime.now().millisecondsSinceEpoch ~/ 1000, title, body, details);
    } catch (e) {
      debugPrint('🔔 $title — $body ($e)');
    }
  }
}
