import 'dart:convert';
import 'dart:io';
import 'package:flutter_secure_storage/flutter_secure_storage.dart';
import 'package:image_picker/image_picker.dart';
import 'package:path_provider/path_provider.dart';
import 'package:path/path.dart' as p;

class UserProfile {
  final String displayName;
  final String? avatarPath;
  final String about;

  UserProfile({
    this.displayName = 'Liberty User',
    this.avatarPath,
    this.about = '',
  });

  Map<String, dynamic> toJson() => {
        'displayName': displayName,
        'avatarPath': avatarPath,
        'about': about,
      };

  factory UserProfile.fromJson(Map<String, dynamic> j) => UserProfile(
        displayName: j['displayName'] as String? ?? 'Liberty User',
        avatarPath: j['avatarPath'] as String?,
        about: j['about'] as String? ?? '',
      );

  UserProfile copyWith({String? displayName, String? avatarPath, String? about}) {
    return UserProfile(
      displayName: displayName ?? this.displayName,
      avatarPath: avatarPath ?? this.avatarPath,
      about: about ?? this.about,
    );
  }
}

class ProfileService {
  static final instance = ProfileService._();
  ProfileService._();

  final _storage = const FlutterSecureStorage();
  UserProfile _profile = UserProfile();

  UserProfile get profile => _profile;

  Future<UserProfile> load() async {
    try {
      final raw = await _storage.read(key: 'profile_v1');
      if (raw != null) {
        _profile = UserProfile.fromJson(jsonDecode(raw) as Map<String, dynamic>);
      }
    } catch (_) {}
    return _profile;
  }

  Future<void> save(UserProfile p) async {
    _profile = p;
    await _storage.write(key: 'profile_v1', value: jsonEncode(p.toJson()));
  }

  Future<String?> pickAvatar() async {
    final x = await ImagePicker().pickImage(source: ImageSource.gallery, maxWidth: 512, imageQuality: 85);
    if (x == null) return null;
    final dir = await getApplicationDocumentsDirectory();
    final dest = File(p.join(dir.path, 'avatar.jpg'));
    await File(x.path).copy(dest.path);
    return dest.path;
  }
}
