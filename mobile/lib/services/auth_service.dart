import 'package:flutter/services.dart';
import 'package:local_auth/local_auth.dart';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';

class AuthService {
  static final AuthService instance = AuthService._();
  AuthService._();

  final _auth = LocalAuthentication();
  final _storage = const FlutterSecureStorage(
    aOptions: AndroidOptions(encryptedSharedPreferences: true),
  );

  static const _kBioEnabled = 'bio_enabled';
  static const _kAutoLockSeconds = 'auto_lock_seconds';

  Future<bool> canUseBiometrics() async {
    try {
      final supported = await _auth.isDeviceSupported();
      final canCheck = await _auth.canCheckBiometrics;
      return supported && canCheck;
    } catch (_) {
      return false;
    }
  }

  Future<List<BiometricType>> availableBiometrics() async {
    try {
      return await _auth.getAvailableBiometrics();
    } catch (_) {
      return [];
    }
  }

  Future<bool> authenticate({String reason = 'Unlock Liberty'}) async {
    try {
      return await _auth.authenticate(
        localizedReason: reason,
        options: const AuthenticationOptions(
          stickyAuth: true,
          biometricOnly: false,
        ),
      );
    } on PlatformException {
      return false;
    }
  }

  Future<void> setBiometricsEnabled(bool enabled) async {
    await _storage.write(key: _kBioEnabled, value: enabled ? '1' : '0');
  }

  Future<bool> isBiometricsEnabled() async {
    return (await _storage.read(key: _kBioEnabled)) == '1';
  }

  Future<void> setAutoLockSeconds(int seconds) async {
    await _storage.write(key: _kAutoLockSeconds, value: '$seconds');
  }

  Future<int> autoLockSeconds() async {
    final v = await _storage.read(key: _kAutoLockSeconds);
    return int.tryParse(v ?? '60') ?? 60;
  }
}
