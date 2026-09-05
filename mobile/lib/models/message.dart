enum MessageStatus { sending, sent, delivered, read, failed }

class ChatMessage {
  final String id;
  final String peerId;
  final String text;
  final bool isMe;
  final DateTime time;
  final MessageStatus status;
  final int ttlSecs;
  final String? mediaPath;
  final String? mediaType;
  final String? encryptedHint;
  final String? replyToId;
  final String? replyToText;
  final int? durationMs;

  ChatMessage({
    required this.id,
    required this.peerId,
    required this.text,
    required this.isMe,
    required this.time,
    this.status = MessageStatus.sent,
    this.ttlSecs = 0,
    this.mediaPath,
    this.mediaType,
    this.encryptedHint,
    this.replyToId,
    this.replyToText,
    this.durationMs,
  });

  ChatMessage copyWith({MessageStatus? status, String? text, String? replyToId, String? replyToText}) {
    return ChatMessage(
      id: id, peerId: peerId, text: text ?? this.text, isMe: isMe, time: time,
      status: status ?? this.status, ttlSecs: ttlSecs, mediaPath: mediaPath, mediaType: mediaType,
      encryptedHint: encryptedHint, replyToId: replyToId ?? this.replyToId,
      replyToText: replyToText ?? this.replyToText, durationMs: durationMs,
    );
  }

  Map<String, dynamic> toJson() => {
        'id': id, 'peerId': peerId, 'text': text, 'isMe': isMe,
        'time': time.toIso8601String(), 'status': status.index, 'ttlSecs': ttlSecs,
        'mediaPath': mediaPath, 'mediaType': mediaType, 'replyToId': replyToId,
        'replyToText': replyToText, 'durationMs': durationMs,
      };

  factory ChatMessage.fromJson(Map<String, dynamic> j) => ChatMessage(
        id: j['id'] as String,
        peerId: j['peerId'] as String,
        text: j['text'] as String? ?? '',
        isMe: j['isMe'] as bool? ?? false,
        time: DateTime.tryParse(j['time'] as String? ?? '') ?? DateTime.now(),
        status: MessageStatus.values[(j['status'] as int?)?.clamp(0, 4) ?? 1],
        ttlSecs: j['ttlSecs'] as int? ?? 0,
        mediaPath: j['mediaPath'] as String?,
        mediaType: j['mediaType'] as String?,
        replyToId: j['replyToId'] as String?,
        replyToText: j['replyToText'] as String?,
        durationMs: j['durationMs'] as int?,
      );
}
