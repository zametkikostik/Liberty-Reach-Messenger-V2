import 'rust_bridge.dart';

enum VaultMode { real, decoy, panic }

class VaultService {
  final _bridge = RustBridge.instance;
  VaultMode? _current;
  bool _ready = false;

  Future<void> init() async {
    if (_ready) return;
    await _bridge.tryLoadNative();
    await _bridge.vaultCreate(
      dbPath: 'vault.db',
      masterPassword: 'master',
      duressPassword: 'duress',
    );
    await _bridge.sessionCreate();
    _ready = true;
  }

  Future<VaultMode> unlock(String password) async {
    await init();
    final result = await _bridge.vaultUnlock(password);
    switch (result) {
      case VaultModeResult.real:
        _current = VaultMode.real;
        return VaultMode.real;
      case VaultModeResult.decoy:
        _current = VaultMode.decoy;
        return VaultMode.decoy;
      case VaultModeResult.panic:
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
    await _bridge.vaultClose();
    _current = null;
  }

  Future<String> peerId() => _bridge.identityCreate('identity.key');
  Future<String> preKeyBundle() => _bridge.getPreKeyBundle();
  Future<String> version() => _bridge.coreVersion();
  bool get isNative => _bridge.isNative;
}
