import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'screens/lock_screen.dart';
import 'screens/chat_list_screen.dart';
import 'services/vault_service.dart';
import 'services/app_lifecycle.dart';
import 'services/platform_security.dart';
import 'services/notification_service.dart';
import 'services/chat_repository.dart';
import 'services/push_service.dart';

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
  PlatformSecurity.setSecureFlag(true);
  ChatRepository.instance.init();
  NotificationService.instance.init();
  PushService.instance.init().then((_) => PushService.instance.loadSavedWorkerUrl());
  runApp(const LibertyApp());
}

class LibertyApp extends StatefulWidget {
  const LibertyApp({super.key});
  @override
  State<LibertyApp> createState() => _LibertyAppState();
}

class _LibertyAppState extends State<LibertyApp> {
  final _navKey = GlobalKey<NavigatorState>();
  final _vault = VaultService();
  AppLifecycleObserver? _lifecycle;

  @override
  void initState() {
    super.initState();
    _lifecycle = AppLifecycleObserver(
      vault: _vault,
      onLocked: () {
        _navKey.currentState?.pushNamedAndRemoveUntil('/', (r) => false);
      },
    );
    _lifecycle!.start();
  }

  @override
  void dispose() {
    _lifecycle?.stop();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      navigatorKey: _navKey,
      title: 'Liberty Messenger',
      debugShowCheckedModeBanner: false,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: const Color(0xFF1A237E),
          brightness: Brightness.dark,
        ),
        useMaterial3: true,
      ),
      home: const LockScreen(),
      routes: {'/chats': (_) => const ChatListScreen()},
    );
  }
}
