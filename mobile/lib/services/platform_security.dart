import 'package:flutter/services.dart';
import 'package:flutter/foundation.dart';
import 'keystore_service.dart';

class PlatformSecurity {
  static const _channel = MethodChannel('liberty/security');

  static Future<void> setSecureFlag(bool enabled) async {
    if (kIsWeb) return;
    try {
      await _channel.invokeMethod('setSecureFlag', {'enabled': enabled});
    } catch (_) {}
  }

  static Future<String> keystoreStatus() async {
    final ok = await KeystoreService.instance.ensureKey();
    if (!ok) return 'unavailable';
    final hw = await KeystoreService.instance.isHardwareBacked();
    return hw ? 'hardware' : 'software';
  }
}
