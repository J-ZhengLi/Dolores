import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

class AttachmentStorage extends StatefulWidget {
  final ChatController chat;
  const AttachmentStorage({super.key, required this.chat});
  @override
  State<AttachmentStorage> createState() => _AttachmentStorageState();
}

class _AttachmentStorageState extends State<AttachmentStorage> {
  bool pending = false;
  String? notice;
  Future<void> cleanup() async {
    final accepted = await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Clean unused attachments?'),
        content: const Text(
          'Remove snapshots no longer referenced by any chat. Current attachments and drafts stay saved.',
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: const Text('Cancel'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: const Text('Clean unused'),
          ),
        ],
      ),
    );
    if (accepted != true || !mounted) return;
    setState(() => pending = true);
    try {
      final result = await widget.chat.bridge.call({
        'command': 'cleanupAttachments',
      }) as Map;
      if (mounted) {
        setState(
          () => notice = 'Removed ${result['removed']} unused snapshots.',
        );
      }
    } catch (_) {
      if (mounted) {
        setState(
          () => notice =
              'Cleanup failed. Referenced attachments are retained. Try again.',
        );
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Attachment storage',
    subtitle: 'Local snapshots',
    canClose: !pending,
    child: Padding(
      padding: const EdgeInsets.all(20),
      child: ListView(
        children: [
          Text(
            'Attachment storage',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 12),
          const Text('Saved chats keep their file and image snapshots.'),
          const SizedBox(height: 16),
          if (notice != null) Text(notice!),
          TextButton(
            onPressed: pending || widget.chat.busy ? null : cleanup,
            child: const Text('Clean unused attachments'),
          ),
        ],
      ),
    ),
  );
}
