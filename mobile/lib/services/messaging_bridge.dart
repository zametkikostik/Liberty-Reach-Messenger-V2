import 'rust_bridge.dart';
import 'network_config.dart';

class MessagingBridge {
  static final instance = MessagingBridge._();
  MessagingBridge._();

  final _bridge = RustBridge.instance;
  int? _sessionHandle;
  String? _peerId;

  Future<void> ensureStarted() async {
    await _bridge.tryLoadNative();
    _sessionHandle ??= await _bridge.sessionCreate();
    _peerId ??= await _bridge.identityCreate('identity.key');
  }

  Future<String> peerId() async {
    await ensureStarted();
    return _peerId ?? '';
  }

  Future<String> preKeyBundle() async {
    await ensureStarted();
    return _bridge.getPreKeyBundle(_sessionHandle);
  }

  Future<String> startSession(String peerId, String bundleJson) async {
    await ensureStarted();
    return _bridge.sessionStartInitiator(peerId, bundleJson);
  }

  Future<String> encrypt(String peerId, String text, {String ephHex = ''}) async {
    await ensureStarted();
    return _bridge.sessionEncrypt(peerId, text, ephHex: ephHex);
  }

  Future<bool> swarmPreferred() => NetworkConfig.instance.swarmEnabled();
}
