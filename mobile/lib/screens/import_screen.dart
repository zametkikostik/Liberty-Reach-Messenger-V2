import 'package:flutter/material.dart';

class ImportScreen extends StatelessWidget {
  const ImportScreen({super.key});
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0D1117),
      appBar: AppBar(backgroundColor: const Color(0xFF161B22), title: const Text('Import')),
      body: const Center(child: Text('Telegram / WhatsApp import (paste export JSON/TXT)', style: TextStyle(color: Colors.white54))),
    );
  }
}
