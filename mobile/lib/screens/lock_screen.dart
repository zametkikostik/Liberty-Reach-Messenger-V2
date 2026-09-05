import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../services/vault_service.dart';
import '../services/auth_service.dart';

class LockScreen extends StatefulWidget {
  const LockScreen({super.key});
  @override
  State<LockScreen> createState() => _LockScreenState();
}

class _LockScreenState extends State<LockScreen> {
  final _controller = TextEditingController();
  final _vault = VaultService();
  bool _loading = false;
  String? _error;
  String _status = '';
  bool _bioEnabled = false;

  @override
  void initState() {
    super.initState();
    _init();
  }

  Future<void> _init() async {
    await _vault.init();
    final ver = await _vault.version();
    final native = _vault.isNative ? 'native' : 'sim';
    final bio = await AuthService.instance.isBiometricsEnabled()
        && await AuthService.instance.canUseBiometrics();
    if (mounted) setState(() { _status = 'core $ver ($native)'; _bioEnabled = bio; });
  }

  Future<void> _tryBiometric() async {
    final ok = await AuthService.instance.authenticate();
    if (!ok || !mounted) return;
    final mode = await _vault.unlock('master');
    if (!mounted) return;
    if (mode == VaultMode.real || mode == VaultMode.decoy) {
      Navigator.of(context).pushReplacementNamed('/chats');
    }
  }

  Future<void> _submit() async {
    if (_loading) return;
    setState(() { _loading = true; _error = null; });
    final mode = await _vault.unlock(_controller.text.trim());
    if (!mounted) return;
    switch (mode) {
      case VaultMode.real:
      case VaultMode.decoy:
        Navigator.of(context).pushReplacementNamed('/chats');
        break;
      case VaultMode.panic:
        setState(() { _error = 'Vault wiped'; _loading = false; _controller.clear(); });
        break;
    }
    setState(() => _loading = false);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 32),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.lock_outline, size: 64, color: Color(0xFF58A6FF)),
              const SizedBox(height: 16),
              const Text('Liberty', style: TextStyle(fontSize: 28, color: Colors.white, fontWeight: FontWeight.bold)),
              const SizedBox(height: 8),
              Text('Sovereign messenger', style: TextStyle(color: Colors.white.withOpacity(0.5))),
              const SizedBox(height: 32),
              TextField(
                controller: _controller,
                obscureText: true,
                style: const TextStyle(color: Colors.white),
                decoration: InputDecoration(
                  filled: true,
                  fillColor: const Color(0xFF161B22),
                  border: OutlineInputBorder(borderRadius: BorderRadius.circular(12), borderSide: BorderSide.none),
                  hintText: 'Password',
                  hintStyle: TextStyle(color: Colors.white.withOpacity(0.3)),
                ),
                onSubmitted: (_) => _submit(),
                inputFormatters: [FilteringTextInputFormatter.deny(RegExp(r'\s'))],
              ),
              if (_error != null) ...[
                const SizedBox(height: 12),
                Text(_error!, style: const TextStyle(color: Colors.redAccent)),
              ],
              const SizedBox(height: 24),
              SizedBox(
                width: double.infinity,
                height: 48,
                child: ElevatedButton(
                  onPressed: _loading ? null : _submit,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: const Color(0xFF238636),
                    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
                  ),
                  child: _loading
                      ? const SizedBox(width: 22, height: 22, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                      : const Text('Unlock', style: TextStyle(color: Colors.white)),
                ),
              ),
              if (_bioEnabled) ...[
                const SizedBox(height: 16),
                IconButton(
                  onPressed: _tryBiometric,
                  icon: const Icon(Icons.fingerprint, size: 40, color: Color(0xFF58A6FF)),
                ),
              ],
              const SizedBox(height: 16),
              Text('Duress password triggers permanent wipe',
                  style: TextStyle(fontSize: 12, color: Colors.white.withOpacity(0.35)), textAlign: TextAlign.center),
              if (_status.isNotEmpty) ...[
                const SizedBox(height: 12),
                Text(_status, style: TextStyle(fontSize: 11, color: Colors.white.withOpacity(0.25))),
              ],
            ],
          ),
        ),
      ),
    );
  }
}
