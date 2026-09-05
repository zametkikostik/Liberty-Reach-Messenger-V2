/// Bridge to liberty_core (Rust).
import 'dart:ffi' as ffi;
import 'package:flutter/foundation.dart';

enum VaultModeResult { real, decoy, panic, error }

class RustBridge {
  static final RustBridge instance = RustBridge._();
  RustBridge._();

  bool _nativeLoaded = false;
  int? _vaultHandle;
  int? _sessionHandle;
  final Map<int, _SimVault> _simVaults = {};
  final Map<int, _SimSession> _simSessions = {};
  int _nextId = 1;

  Future<bool> tryLoadNative() async {
    if (kIsWeb) return false;
    try {
      for (final name in ['libliberty_core.so', 'liberty_core', 'libliberty_core.dylib']) {
        try {
          ffi.DynamicLibrary.open(name);
          _nativeLoaded = true;
          return true;
        } catch (_) {}
      }
    } catch (e) {
      debugPrint('RustBridge: $e');
    }
    debugPrint('RustBridge: simulation mode');
    return false;
  }

  bool get isNative => _nativeLoaded;

  Future<bool> vaultCreate({required String dbPath, required String masterPassword, required String duressPassword}) async {
    final id = _nextId++;
    _simVaults[id] = _SimVault(masterPassword, duressPassword);
    _vaultHandle = id;
    return true;
  }

  Future<VaultModeResult> vaultUnlock(String password) async {
    if (_vaultHandle == null) return VaultModeResult.error;
    final v = _simVaults[_vaultHandle];
    if (v == null) return VaultModeResult.error;
    if (password == v.duress) {
      _simVaults.remove(_vaultHandle);
      _vaultHandle = null;
      return VaultModeResult.panic;
    }
    if (password == v.master) {
      v.mode = VaultModeResult.real;
      return VaultModeResult.real;
    }
    v.mode = VaultModeResult.decoy;
    return VaultModeResult.decoy;
  }

  Future<void> vaultClose() async {
    if (_vaultHandle != null) {
      _simVaults.remove(_vaultHandle);
      _vaultHandle = null;
    }
  }

  Future<bool> vaultIsReal() async {
    if (_vaultHandle == null) return false;
    return _simVaults[_vaultHandle]?.mode == VaultModeResult.real;
  }

  Future<String> identityCreate(String path) async =>
      'peer-${DateTime.now().millisecondsSinceEpoch % 100000}';

  Future<int> sessionCreate() async {
    final id = _nextId++;
    _simSessions[id] = _SimSession();
    _sessionHandle = id;
    return id;
  }

  Future<String> getPreKeyBundle([int? handle]) async {
    final h = handle ?? _sessionHandle;
    if (h == null) return '';
    return '{"sim":true,"handle":$h,"identity_ed25519":"00","signed_prekey":"00"}';
  }

  Future<String> sessionStartInitiator(String peerId, String theirBundleJson) async {
    final h = _sessionHandle;
    if (h == null) return '';
    _simSessions[h]?.peers.add(peerId);
    return List.filled(32, 'ab').join();
  }

  Future<String> sessionEncrypt(String peerId, String plaintext, {String ephHex = ''}) async {
    final h = _sessionHandle;
    if (h == null) return '';
    return '{"header":{"n":0},"ciphertext":"${plaintext.hashCode.toRadixString(16)}","timestamp":${DateTime.now().millisecondsSinceEpoch ~/ 1000}}';
  }

  Future<String> coreVersion() async => _nativeLoaded ? '0.1.0-native' : '0.1.0-sim';
}

class _SimVault {
  final String master;
  final String duress;
  VaultModeResult? mode;
  _SimVault(this.master, this.duress);
}

class _SimSession {
  final peers = <String>{};
}
