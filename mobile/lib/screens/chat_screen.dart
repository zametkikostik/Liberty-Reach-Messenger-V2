import 'dart:async';
import 'dart:io';
import 'package:flutter/material.dart';
import '../models/message.dart';
import '../services/chat_repository.dart';
import '../services/media_service.dart';
import '../services/typing_service.dart';
import '../services/voice_service.dart';

class ChatScreen extends StatefulWidget {
  final String peerName;
  final String peerId;
  final bool isRealMode;
  const ChatScreen({super.key, required this.peerName, required this.peerId, required this.isRealMode});
  @override
  State<ChatScreen> createState() => _ChatScreenState();
}

class _ChatScreenState extends State<ChatScreen> {
  final _controller = TextEditingController();
  final _repo = ChatRepository.instance;
  final _scroll = ScrollController();
  StreamSubscription? _sub;
  List<ChatMessage> _messages = [];
  bool _sending = false;

  @override
  void initState() {
    super.initState();
    _repo.init();
    _messages = _repo.messagesFor(widget.peerId);
    _sub = _repo.watchChat(widget.peerId).listen((list) {
      if (mounted) setState(() => _messages = list);
    });
    _repo.markRead(widget.peerId);
  }

  Future<void> _send() async {
    final text = _controller.text.trim();
    if (text.isEmpty || _sending || !widget.isRealMode) return;
    setState(() => _sending = true);
    _controller.clear();
    await _repo.sendText(peerId: widget.peerId, text: text);
    if (mounted) setState(() => _sending = false);
  }

  @override
  void dispose() {
    _sub?.cancel();
    _controller.dispose();
    _scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(
        backgroundColor: const Color(0xFF161B22),
        title: Text(widget.peerName),
      ),
      body: Column(
        children: [
          Expanded(
            child: ListView.builder(
              controller: _scroll,
              padding: const EdgeInsets.all(12),
              itemCount: _messages.length,
              itemBuilder: (_, i) {
                final m = _messages[i];
                return Align(
                  alignment: m.isMe ? Alignment.centerRight : Alignment.centerLeft,
                  child: Container(
                    margin: const EdgeInsets.symmetric(vertical: 4),
                    padding: const EdgeInsets.all(10),
                    decoration: BoxDecoration(
                      color: m.isMe ? const Color(0xFF238636) : const Color(0xFF21262D),
                      borderRadius: BorderRadius.circular(16),
                    ),
                    child: Text(m.text.isEmpty ? (m.mediaType ?? '') : m.text, style: const TextStyle(color: Colors.white)),
                  ),
                );
              },
            ),
          ),
          if (widget.isRealMode)
            Container(
              color: const Color(0xFF161B22),
              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 6),
              child: Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _controller,
                      style: const TextStyle(color: Colors.white),
                      decoration: const InputDecoration(hintText: 'Message', border: InputBorder.none, hintStyle: TextStyle(color: Colors.white38)),
                      onSubmitted: (_) => _send(),
                    ),
                  ),
                  IconButton(icon: const Icon(Icons.send, color: Color(0xFF58A6FF)), onPressed: _sending ? null : _send),
                ],
              ),
            ),
        ],
      ),
    );
  }
}
