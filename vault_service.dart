import 'dart:io' show Platform;

import 'package:path_provider/path_provider.dart';

import 'rust_bridge.dart';

enum VaultMode { real, decoy, panic }

/// Hidden-vault lifecycle. Phase 1 keeps the demo credentials from the
/// README (`master` / `duress` / anything else = decoy); proper onboarding
/// with user-chosen passwords is the next product step (see PATCH_NOTES).
class VaultService {
  static final VaultService instance = VaultService._();
  VaultService._();

  final _bridge = RustBridge.instance;
  VaultMode? _current;
  bool _ready = false;
  String? _dbPath;
  String? _identityPath;
  String? _sessionStatePath;

  Future<void> init() async {
    if (_ready) return;
    final loaded = await _bridge.tryLoadNative();
    if (!loaded) {
      throw StateError(
        'Native core (libliberty_core.so) not found. '
        'Run scripts/build_android_native.sh before running the app.',
      );
    }
    // Absolute, writable paths: a relative 'vault.db' would resolve to '/'
    // on Android and make every unlock fail.
    final dir = await getApplicationDocumentsDirectory();
    final sep = Platform.pathSeparator;
    _dbPath = '${dir.path}$sepvault.db';
    _identityPath = '${dir.path}$sepidentity.key';
    _sessionStatePath = '${dir.path}$sepsession_state.bin';
    await _bridge.vaultCreate(
      dbPath: _dbPath!,
      masterPassword: 'master',
      duressPassword: 'duress',
    );
    await _bridge.sessionCreate(identityPath: _identityPath);
    _ready = true;
  }

  Future<VaultMode> unlock(String password) async {
    await init();
    final result = await _bridge.vaultUnlock(password);
    switch (result) {
      case VaultModeResult.real:
        _current = VaultMode.real;
        // Restore conversations from the encrypted session state, then keep
        // it auto-persisted after every encrypt/decrypt (survives kills).
        await _bridge.sessionLoad(_sessionStatePath!, password);
        await _bridge.sessionSetAutosave(_sessionStatePath!, password);
        return VaultMode.real;
      case VaultModeResult.decoy:
        // Decoy mode never touches the real session state.
        _current = VaultMode.decoy;
        return VaultMode.decoy;
      case VaultModeResult.panic:
        // Rust already wiped registered autosave files + the identity file;
        // wipe the state path explicitly in case nothing was registered yet.
        await _bridge.sessionWipeState(_sessionStatePath!);
        _current = null;
        return VaultMode.panic;
      case VaultModeResult.error:
        _current = null;
        return VaultMode.panic;
    }
  }

  Future<VaultMode?> currentMode() async {
    if (_current != null) return _current;
    if (await _bridge.vaultIsReal()) {
      _current = VaultMode.real;
      return VaultMode.real;
    }
    return _current;
  }

  Future<void> lock() async {
    // Stop autosaving before closing: a later decoy unlock must not write
    // into the real session-state file.
    await _bridge.sessionClearAutosave();
    await _bridge.vaultClose();
    _current = null;
  }

  String get identityPath => _identityPath ?? 'identity.key';
  String get sessionStatePath => _sessionStatePath ?? 'session_state.bin';
  Future<String> peerId() => _bridge.identityCreate(identityPath);
  Future<String> preKeyBundle() => _bridge.getPreKeyBundle();
  Future<String> version() => _bridge.coreVersion();
  bool get isNative => _bridge.isNative;
}
