/// Bridge to liberty_core (Rust) — REAL FFI calls, no simulation.
///
/// If the native library (`libliberty_core.so`) is missing, `tryLoadNative`
/// returns false and every operation throws a [StateError]: the app must
/// fail loudly rather than silently fall back to fake crypto.
///
/// Memory contract: every string returned by the C ABI is copied into Dart
/// and freed on the Rust side via `liberty_free_string`; every argument is
/// allocated with [calloc] and freed after the call.
import 'dart:ffi' as ffi;

import 'package:ffi/ffi.dart';

enum VaultModeResult { real, decoy, panic, error }

class RustBridge {
  static final RustBridge instance = RustBridge._();
  RustBridge._();

  static const List<String> _libNames = [
    'libliberty_core.so', // Android / Linux
    'libliberty_core.dylib', // macOS / iOS
    'libliberty_core', // bare (some linkers)
    'liberty_core', // windows-style fallback
  ];

  ffi.DynamicLibrary? _lib;
  int? _vaultHandle;
  int? _sessionHandle;

  bool get isNative => _lib != null;

  /// Try to load the native library. True on success.
  Future<bool> tryLoadNative() async {
    if (_lib != null) return true;
    for (final name in _libNames) {
      try {
        _lib = ffi.DynamicLibrary.open(name);
        return true;
      } catch (_) {}
    }
    return false;
  }

  ffi.DynamicLibrary get _dl {
    final lib = _lib;
    if (lib == null) {
      throw StateError(
        'liberty_core native library is not loaded — build it with '
        'scripts/build_android_native.sh (see PATCH_NOTES).',
      );
    }
    return lib;
  }

  // --------------------------------------------------------------------
  // Low-level call helpers (one per C arity used by the ABI)
  // --------------------------------------------------------------------

  /// Take ownership of a returned C string; empty string for null.
  String _take(ffi.Pointer<ffi.Uint8> p) {
    if (p == ffi.nullptr) return '';
    final s = p.cast<Utf8>().toDartString();
    _dl
        .lookupFunction<
            ffi.Void Function(ffi.Pointer<ffi.Uint8>),
            void Function(ffi.Pointer<ffi.Uint8>)>('liberty_free_string')(p);
    return s;
  }

  String _g_str(String name) => _take(
      _dl.lookupFunction<ffi.Pointer<ffi.Uint8> Function(),
              ffi.Pointer<ffi.Uint8> Function()>(name)());

  int _g_u32(String name) =>
      _dl.lookupFunction<ffi.Uint32 Function(), int Function()>(name)();

  bool _g_bool(String name) =>
      _dl.lookupFunction<ffi.Bool Function(), bool Function()>(name)();

  String _c1_str(String name, String a) {
    final pa = a.toNativeUtf8(allocator: calloc);
    try {
      return _take(_dl.lookupFunction<
          ffi.Pointer<ffi.Uint8> Function(ffi.Pointer<ffi.Uint8>),
          ffi.Pointer<ffi.Uint8> Function(
              ffi.Pointer<ffi.Uint8>)>(name)(pa));
    } finally {
      calloc.free(pa);
    }
  }

  int _c1_u32(String name, String a) {
    final pa = a.toNativeUtf8(allocator: calloc);
    try {
      return _dl.lookupFunction<
          ffi.Uint32 Function(ffi.Pointer<ffi.Uint8>),
          int Function(ffi.Pointer<ffi.Uint8>)>(name)(pa);
    } finally {
      calloc.free(pa);
    }
  }

  bool _c1_bool(String name, String a) {
    final pa = a.toNativeUtf8(allocator: calloc);
    try {
      return _dl.lookupFunction<
          ffi.Bool Function(ffi.Pointer<ffi.Uint8>),
          bool Function(ffi.Pointer<ffi.Uint8>)>(name)(pa);
    } finally {
      calloc.free(pa);
    }
  }

  int _c3_u32(String name, String a, String b, String c) {
    final pa = a.toNativeUtf8(allocator: calloc);
    final pb = b.toNativeUtf8(allocator: calloc);
    final pc = c.toNativeUtf8(allocator: calloc);
    try {
      return _dl.lookupFunction<
          ffi.Uint32 Function(ffi.Pointer<ffi.Uint8>, ffi.Pointer<ffi.Uint8>,
              ffi.Pointer<ffi.Uint8>),
          int Function(ffi.Pointer<ffi.Uint8>, ffi.Pointer<ffi.Uint8>,
              ffi.Pointer<ffi.Uint8>)>(name)(pa, pb, pc);
    } finally {
      calloc.free(pa);
      calloc.free(pb);
      calloc.free(pc);
    }
  }

  int _c2_i32_h(String name, int handle, String a) {
    final pa = a.toNativeUtf8(allocator: calloc);
    try {
      return _dl.lookupFunction<
          ffi.Int32 Function(ffi.Uint32, ffi.Pointer<ffi.Uint8>),
          int Function(int, ffi.Pointer<ffi.Uint8>)>(name)(handle, pa);
    } finally {
      calloc.free(pa);
    }
  }

  String _c3_h_str(String name, int handle, String a, String b) {
    final pa = a.toNativeUtf8(allocator: calloc);
    final pb = b.toNativeUtf8(allocator: calloc);
    try {
      return _take(_dl.lookupFunction<
          ffi.Pointer<ffi.Uint8> Function(
              ffi.Uint32, ffi.Pointer<ffi.Uint8>, ffi.Pointer<ffi.Uint8>),
          ffi.Pointer<ffi.Uint8> Function(
              int, ffi.Pointer<ffi.Uint8>, ffi.Pointer<ffi.Uint8>)>(name)(
          handle, pa, pb));
    } finally {
      calloc.free(pa);
      calloc.free(pb);
    }
  }

  String _c4_h_str(String name, int handle, String a, String b, String c) {
    final pa = a.toNativeUtf8(allocator: calloc);
    final pb = b.toNativeUtf8(allocator: calloc);
    final pc = c.toNativeUtf8(allocator: calloc);
    try {
      return _take(_dl.lookupFunction<
          ffi.Pointer<ffi.Uint8> Function(ffi.Uint32, ffi.Pointer<ffi.Uint8>,
              ffi.Pointer<ffi.Uint8>, ffi.Pointer<ffi.Uint8>),
          ffi.Pointer<ffi.Uint8> Function(int, ffi.Pointer<ffi.Uint8>,
              ffi.Pointer<ffi.Uint8>, ffi.Pointer<ffi.Uint8>)>(name)(
          handle, pa, pb, pc));
    } finally {
      calloc.free(pa);
      calloc.free(pb);
      calloc.free(pc);
    }
  }

  String _c_h_str(String name, int handle) => _take(_dl.lookupFunction<
          ffi.Pointer<ffi.Uint8> Function(ffi.Uint32),
          ffi.Pointer<ffi.Uint8> Function(int)>(name)(handle));

  bool _c_h_bool(String name, int handle) =>
      _dl.lookupFunction<ffi.Bool Function(ffi.Uint32), bool Function(int)>(
          name)(handle);

  bool _c2_h_bool(String name, int handle, String a) {
    final pa = a.toNativeUtf8(allocator: calloc);
    try {
      return _dl.lookupFunction<
          ffi.Bool Function(ffi.Uint32, ffi.Pointer<ffi.Uint8>),
          bool Function(int, ffi.Pointer<ffi.Uint8>)>(name)(handle, pa);
    } finally {
      calloc.free(pa);
    }
  }

  // --------------------------------------------------------------------
  // Public API (kept close to the previous surface; simulation removed)
  // --------------------------------------------------------------------

  Future<String> coreVersion() async => _g_str('liberty_core_version');
  Future<String> buildFeatures() async => _g_str('liberty_build_features');
  Future<String> cipherSuites() async => _g_str('liberty_cipher_suites');

  Future<bool> vaultCreate({
    required String dbPath,
    required String masterPassword,
    required String duressPassword,
  }) async {
    final h = _c3_u32(
        'liberty_vault_create', dbPath, masterPassword, duressPassword);
    _vaultHandle = h;
    return h != 0;
  }

  Future<VaultModeResult> vaultUnlock(String password) async {
    final h = _vaultHandle;
    if (h == null || h == 0) return VaultModeResult.error;
    switch (_c2_i32_h('liberty_vault_unlock', h, password)) {
      case 1:
        return VaultModeResult.real;
      case 2:
        return VaultModeResult.decoy;
      case 3:
        return VaultModeResult.panic;
      default:
        return VaultModeResult.error;
    }
  }

  Future<void> vaultClose() async {
    final h = _vaultHandle;
    if (h == null || h == 0) return;
    _c_h_bool('liberty_vault_close', h);
    _vaultHandle = null;
  }

  Future<bool> vaultIsReal() async {
    final h = _vaultHandle;
    if (h == null || h == 0) return false;
    return _c_h_bool('liberty_vault_is_real', h);
  }

  /// Load-or-generate identity file; returns the peer id.
  Future<String> identityCreate(String path) async =>
      _c1_str('liberty_identity_create', path);

  /// Create a session. Pass [identityPath] to bind it to the persisted
  /// identity (stable prekey bundle across restarts).
  Future<int> sessionCreate({String? identityPath}) async {
    final h = identityPath == null
        ? _g_u32('liberty_session_create')
        : _c1_u32('liberty_session_create_with_identity', identityPath);
    _sessionHandle = h;
    return h;
  }

  Future<String> getPreKeyBundle([int? handle]) async {
    final h = handle ?? _sessionHandle;
    if (h == null || h == 0) return '';
    return _c_h_str('liberty_session_prekey_bundle', h);
  }

  /// Start a session as initiator. Returns OUR ephemeral public key (hex);
  /// keep it with the contact and pass to [sessionEncrypt] as `ephHex`.
  Future<String> sessionStartInitiator(
      String peerId, String theirBundleJson) async {
    final h = _sessionHandle;
    if (h == null || h == 0) return '';
    final eph = _c3_h_str(
        'liberty_session_start_initiator', h, peerId, theirBundleJson);
    return eph;
  }

  /// Encrypt [plaintext] for [peerId].
  /// [ephHex] = OUR initiator ephemeral (hex) for that peer; '' if responding.
  /// Returns EncryptedMessage JSON, or '' on failure.
  Future<String> sessionEncrypt(String peerId, String plaintext,
      {String ephHex = ''}) async {
    final h = _sessionHandle;
    if (h == null || h == 0) return '';
    return _c4_h_str('liberty_session_encrypt', h, peerId, plaintext, ephHex);
  }

  /// Decrypt an incoming EncryptedMessage JSON. Returns plaintext,
  /// or '' on failure (bootstraps a responder session on first message).
  Future<String> sessionDecrypt(String peerId, String encryptedJson) async {
    final h = _sessionHandle;
    if (h == null || h == 0) return '';
    return _c3_h_str('liberty_session_decrypt', h, peerId, encryptedJson);
  }

  Future<bool> sessionHas(String peerId) async {
    final h = _sessionHandle;
    if (h == null || h == 0) return false;
    return _c2_h_bool('liberty_session_has', h, peerId);
  }

  // Federation passthroughs
  Future<String> federationGet() async => _g_str('liberty_federation_get');
  Future<bool> federationSetJson(String json) async =>
      _c1_bool('liberty_federation_set_json', json);
  Future<String> federationBootstrapAddrs() async =>
      _g_str('liberty_federation_bootstrap_addrs');
  Future<bool> federationMeshEnabled() async =>
      _g_bool('liberty_federation_mesh_enabled');
}
