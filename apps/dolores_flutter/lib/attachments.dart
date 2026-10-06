import 'dart:convert';
import 'dart:typed_data';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'chat.dart';

Future<void> chooseAttachment(ChatController chat) async {
  String? path;
  try {
    await chat.inspectLocalSettings(() async {
      path = (await openFile())?.path;
    });
    if (path != null) await chat.addAttachment(path!);
  } catch (failure) {
    chat.reportLocalError(
      'Could not open the attachment picker. Your draft remains. $failure',
    );
  }
}

Future<void> previewAttachment(
  BuildContext context,
  ChatController chat,
  Map part,
) async {
  try {
    final preview = await chat.bridge.call({
      'command': 'attachmentPreview',
      'session': chat.session,
      'digest': part['digest'],
    }) as Map;
    if (!context.mounted) return;
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(part['name'] as String),
        content: SizedBox(
          width: 560,
          height: 380,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                '${part['mime']} · ${part['bytes']} bytes\nLocal immutable snapshot. Sending shares the included content with your configured provider.',
              ),
              const SizedBox(height: 12),
              Expanded(
                child: preview['imageBase64'] is String
                    ? Image.memory(
                        base64Decode(preview['imageBase64'] as String),
                        fit: BoxFit.contain,
                        errorBuilder: (_, _, _) => const Text(
                          'Image preview could not be decoded. Remove it and choose another image.',
                        ),
                      )
                    : SingleChildScrollView(
                        child: SelectableText(
                          '${preview['text']}${preview['previewTruncated'] == true ? '\n\nPreview shortened to 8 KiB. The complete snapshot is shared when sent.' : ''}',
                        ),
                      ),
              ),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Close'),
          ),
        ],
      ),
    );
  } catch (failure) {
    if (context.mounted) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(failure.toString())));
    }
  }
}

class AttachmentChips extends StatelessWidget {
  final ChatController chat;
  final List<Map<String, dynamic>> parts;
  final bool removable;
  const AttachmentChips({
    super.key,
    required this.chat,
    required this.parts,
    this.removable = false,
  });
  @override
  Widget build(BuildContext context) => Wrap(
    spacing: 6,
    runSpacing: 4,
    children: [
      for (final part in parts)
        if ((part['mime'] as String).startsWith('image/'))
          AttachmentThumbnail(
            key: ValueKey('${chat.session}:${part['digest']}'),
            chat: chat,
            part: part,
            removable: removable,
          )
        else
          InputChip(
            avatar: Icon(
              (part['mime'] as String).startsWith('image/')
                  ? Icons.image_outlined
                  : Icons.description_outlined,
              size: 16,
            ),
            label: SizedBox(
              width: 160,
              child: Text(
                part['name'] as String,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            tooltip:
                '${part['mime']} · ${part['bytes']} bytes · Click to inspect sharing',
            onPressed: () => previewAttachment(context, chat, part),
            onDeleted:
                removable && !chat.busy && !chat.changing && !chat.loading
                ? () => chat.removeAttachment(part['digest'] as String)
                : null,
          ),
    ],
  );
}

class AttachmentThumbnail extends StatefulWidget {
  final ChatController chat;
  final Map<String, dynamic> part;
  final bool removable;
  const AttachmentThumbnail({
    super.key,
    required this.chat,
    required this.part,
    this.removable = false,
  });
  @override
  State<AttachmentThumbnail> createState() => _AttachmentThumbnailState();
}

class _AttachmentThumbnailState extends State<AttachmentThumbnail> {
  late Future<Uint8List> image;
  @override
  void initState() {
    super.initState();
    image = load();
  }

  Future<Uint8List> load() async {
    final preview = await widget.chat.bridge.call({
      'command': 'attachmentPreview',
      'session': widget.chat.session,
      'digest': widget.part['digest'],
    }) as Map;
    final bytes = base64Decode(preview['imageBase64'] as String);
    if (bytes.isEmpty || bytes.length > 2 * 1024 * 1024) {
      throw StateError('Preview unavailable');
    }
    return bytes;
  }

  @override
  Widget build(BuildContext context) => SizedBox(
    width: 144,
    height: 112,
    child: Stack(
      children: [
        Positioned.fill(
          child: ClipRRect(
            borderRadius: BorderRadius.circular(10),
            child: Material(
              color: Theme.of(context).colorScheme.surfaceContainerHigh,
              child: FutureBuilder<Uint8List>(
                future: image,
                builder: (context, snap) {
                  if (snap.hasError) {
                    return Center(
                      child: TextButton.icon(
                        onPressed: () => setState(() => image = load()),
                        icon: const Icon(Icons.refresh, size: 16),
                        label: const Text('Retry preview'),
                      ),
                    );
                  }
                  return InkWell(
                    onTap: () =>
                        previewAttachment(context, widget.chat, widget.part),
                    child: Tooltip(
                      message: widget.part['name'] as String,
                      child: snap.hasData
                          ? Image.memory(
                              snap.data!,
                              fit: BoxFit.contain,
                              cacheWidth: 288,
                              semanticLabel: widget.part['name'] as String,
                              errorBuilder: (_, _, _) =>
                                  const Center(child: Text('Invalid image')),
                            )
                          : const Center(child: Icon(Icons.image_outlined)),
                    ),
                  );
                },
              ),
            ),
          ),
        ),
        if (widget.removable)
          Positioned(
            top: 0,
            right: 0,
            child: IconButton.filledTonal(
              visualDensity: VisualDensity.compact,
              iconSize: 16,
              tooltip: 'Remove image',
              icon: const Icon(Icons.close),
              onPressed:
                  widget.chat.busy ||
                      widget.chat.changing ||
                      widget.chat.loading
                  ? null
                  : () => widget.chat.removeAttachment(
                      widget.part['digest'] as String,
                    ),
            ),
          ),
      ],
    ),
  );
}

Future<void> exportAttachments(
  BuildContext context,
  ChatController chat,
) async {
  try {
    await chat.inspectLocalSettings(() async {
      final directory = await getDirectoryPath();
      if (directory == null || chat.session == null) return;
      final result = await chat.bridge.call({
        'command': 'exportAttachments',
        'session': chat.session,
        'directory': directory,
      }) as Map;
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              'Exported ${result['count']} snapshots to ${result['folder']}',
            ),
          ),
        );
      }
    });
  } catch (failure) {
    if (context.mounted) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text(failure.toString())));
    }
  }
}
