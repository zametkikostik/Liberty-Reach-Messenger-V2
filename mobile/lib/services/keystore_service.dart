import 'dart:typed_data';
import 'package:flutter/services.dart';
import 'package:flutter/foundation.dart';

class KeystoreService {
  static final instance = KeystoreService._();
  KeystoreService._();

  static const _channel = MethodChannel('liberty/security');

  Future<bool> ensureKey() async {
    if (kIsWeb) return false;
    try {
      return await _channel.invokeMethod<bool>('keystoreEnsure') ?? false;
    } catch (_) {
      return false;
    }
  }

  Future<String?> wrap(Uint8List data) async {
    try {
      return await _channel.invokeMethod<String>('keystoreWrap', {'data': data});
    } catch (_) {
      return null;
    }
  }

  Future<Uint8List?> unwrap(String wrappedB64) async {
    try {
      final result = await _channel.invokeMethod('keystoreUnwrap', {'wrapped': wrappedB64});
      if (result is Uint8List) return result;
      if (result is List) return Uint8List.fromList(result.cast<int>());
      return null;
    } catch (_) {
      return null;
    }
  }

  Future<void> deleteKey() async {
    try {
      await _channel.invokeMethod('keystoreDelete');
    } catch (_) {}
  }

  Future<bool> isHardwareBacked() async {
    try {
      return await _channel.invokeMethod<bool>('keystoreIsHardware') ?? false;
    } catch (_) {
      return false;
    }
  }
}
