import 'dart:async';
import 'package:flutter/widgets.dart';
import 'auth_service.dart';
import 'vault_service.dart';
import 'push_service.dart';

class AppLifecycleObserver with WidgetsBindingObserver {
  final VaultService vault;
  final VoidCallback onLocked;
  Timer? _timer;
  DateTime? _pausedAt;

  AppLifecycleObserver({required this.vault, required this.onLocked});

  void start() {
    WidgetsBinding.instance.addObserver(this);
  }

  void stop() {
    WidgetsBinding.instance.removeObserver(this);
    _timer?.cancel();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (state == AppLifecycleState.paused || state == AppLifecycleState.inactive) {
      _pausedAt = DateTime.now();
      _scheduleLock();
    } else if (state == AppLifecycleState.resumed) {
      _timer?.cancel();
      _checkLockOnResume();
      PushService.instance.poll();
    }
  }

  Future<void> _scheduleLock() async {
    final seconds = await AuthService.instance.autoLockSeconds();
    if (seconds <= 0) return;
    _timer?.cancel();
    _timer = Timer(Duration(seconds: seconds), () async {
      await vault.lock();
      onLocked();
    });
  }

  Future<void> _checkLockOnResume() async {
    final seconds = await AuthService.instance.autoLockSeconds();
    if (seconds <= 0 || _pausedAt == null) return;
    final elapsed = DateTime.now().difference(_pausedAt!).inSeconds;
    if (elapsed >= seconds) {
      await vault.lock();
      onLocked();
    }
  }
}
