import 'dart:io';
import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;
import 'package:record/record.dart';
import 'package:audioplayers/audioplayers.dart';
import 'package:permission_handler/permission_handler.dart';

class VoiceService {
  static final instance = VoiceService._();
  VoiceService._();

  final _recorder = AudioRecorder();
  final _player = AudioPlayer();
  DateTime? _started;
  String? _currentPath;
  String? _playingId;

  bool get isRecording => _started != null;
  String? get playingId => _playingId;

  Future<bool> _ensureMic() async {
    final status = await Permission.microphone.request();
    return status.isGranted;
  }

  Future<void> start() async {
    if (!await _ensureMic()) return;
    if (!await _recorder.hasPermission()) return;
    final dir = await getApplicationDocumentsDirectory();
    final voiceDir = Directory(p.join(dir.path, 'voice'));
    if (!await voiceDir.exists()) await voiceDir.create(recursive: true);
    _currentPath = p.join(voiceDir.path, '${DateTime.now().millisecondsSinceEpoch}.m4a');
    await _recorder.start(
      const RecordConfig(encoder: AudioEncoder.aacLc, bitRate: 128000, sampleRate: 44100),
      path: _currentPath!,
    );
    _started = DateTime.now();
  }

  Future<({String path, int durationMs})?> stop() async {
    if (_started == null) return null;
    final ms = DateTime.now().difference(_started!).inMilliseconds;
    _started = null;
    final path = await _recorder.stop();
    final finalPath = path ?? _currentPath;
    _currentPath = null;
    if (finalPath == null || ms < 400) {
      if (finalPath != null) { try { await File(finalPath).delete(); } catch (_) {} }
      return null;
    }
    return (path: finalPath, durationMs: ms);
  }

  void cancel() async {
    _started = null;
    try { await _recorder.stop(); } catch (_) {}
    if (_currentPath != null) {
      try { await File(_currentPath!).delete(); } catch (_) {}
      _currentPath = null;
    }
  }

  Future<void> play(String messageId, String path) async {
    if (_playingId == messageId) { await stopPlayback(); return; }
    await stopPlayback();
    _playingId = messageId;
    await _player.play(DeviceFileSource(path));
    _player.onPlayerComplete.listen((_) { _playingId = null; });
  }

  Future<void> stopPlayback() async {
    await _player.stop();
    _playingId = null;
  }

  Future<void> dispose() async {
    await _recorder.dispose();
    await _player.dispose();
  }
}
