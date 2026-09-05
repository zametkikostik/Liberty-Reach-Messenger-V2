import 'dart:async';

/// Ephemeral typing indicators (not persisted, not E2EE-critical).
class TypingService {
  static final TypingService instance = TypingService._();
  TypingService._();

  final _local = <String, bool>{};
  final _remote = <String, bool>{};
  final _controllers = <String, StreamController<bool>>{};
  final _stopTimers = <String, Timer>{};

  Stream<bool> watch(String peerId) {
    _controllers.putIfAbsent(
      peerId,
      () => StreamController<bool>.broadcast(),
    );
    return _controllers[peerId]!.stream;
  }

  void setLocalTyping(String peerId, bool typing) {
    _local[peerId] = typing;
  }

  void setRemoteTyping(String peerId, bool typing) {
    _remote[peerId] = typing;
    _controllers[peerId]?.add(typing);
    _stopTimers[peerId]?.cancel();
    if (typing) {
      _stopTimers[peerId] = Timer(const Duration(seconds: 3), () {
        _remote[peerId] = false;
        _controllers[peerId]?.add(false);
      });
    }
  }

  bool isRemoteTyping(String peerId) => _remote[peerId] ?? false;
}
