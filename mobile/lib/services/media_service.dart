import 'dart:io';
import 'dart:typed_data';
import 'package:file_picker/file_picker.dart';
import 'package:image_picker/image_picker.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;
import 'package:flutter/foundation.dart';

class PickedMedia {
  final String path;
  final String filename;
  final String mime;
  final int size;
  final Uint8List bytes;

  PickedMedia({
    required this.path,
    required this.filename,
    required this.mime,
    required this.size,
    required this.bytes,
  });
}

class MediaService {
  static final instance = MediaService._();
  MediaService._();

  final _images = ImagePicker();

  Future<PickedMedia?> pickImage({bool fromCamera = false}) async {
    try {
      final x = await _images.pickImage(
        source: fromCamera ? ImageSource.camera : ImageSource.gallery,
        imageQuality: 85,
        maxWidth: 1920,
      );
      if (x == null) return null;
      final bytes = await x.readAsBytes();
      return PickedMedia(
        path: x.path,
        filename: p.basename(x.path),
        mime: 'image/jpeg',
        size: bytes.length,
        bytes: bytes,
      );
    } catch (e) {
      debugPrint('pickImage: $e');
      return null;
    }
  }

  Future<PickedMedia?> pickFile() async {
    try {
      final r = await FilePicker.platform.pickFiles(withData: true);
      if (r == null || r.files.isEmpty) return null;
      final f = r.files.first;
      final bytes = f.bytes ?? (f.path != null ? await File(f.path!).readAsBytes() : null);
      if (bytes == null) return null;
      return PickedMedia(
        path: f.path ?? f.name,
        filename: f.name,
        mime: f.extension != null ? 'application/${f.extension}' : 'application/octet-stream',
        size: bytes.length,
        bytes: Uint8List.fromList(bytes),
      );
    } catch (e) {
      debugPrint('pickFile: $e');
      return null;
    }
  }

  Future<String> saveOutbound(String peerId, PickedMedia media) async {
    final dir = await getApplicationDocumentsDirectory();
    final mediaDir = Directory(p.join(dir.path, 'media', peerId));
    if (!await mediaDir.exists()) await mediaDir.create(recursive: true);
    final name = '${DateTime.now().millisecondsSinceEpoch}_${media.filename}';
    final file = File(p.join(mediaDir.path, name));
    await file.writeAsBytes(media.bytes);
    return file.path;
  }
}
