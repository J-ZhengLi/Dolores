import 'form_field_layout.dart';

import 'dart:convert';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'bridge.dart';
import 'settings_frame.dart';

class SkillCreateDialog extends StatefulWidget {
  final ChatBridge bridge;
  final String session, scope;
  final bool importing;
  const SkillCreateDialog({
    super.key,
    required this.bridge,
    required this.session,
    required this.scope,
    this.importing = false,
  });
  @override
  State<SkillCreateDialog> createState() => _SkillCreateState();
}

class _SkillCreateState extends State<SkillCreateDialog> {
  final name = TextEditingController(),
      description = TextEditingController(),
      body = TextEditingController();
  bool pending = false;
  String? error;
  Future<void> close() async {
    if (pending) return;
    if (name.text.isNotEmpty ||
        description.text.isNotEmpty ||
        body.text.isNotEmpty) {
      final discard = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Discard skill draft?'),
          content: const Text('Your skill has not been saved.'),
          actions: [
            TextButton(
              onPressed: () => Navigator.pop(context, false),
              child: const Text('Keep editing'),
            ),
            TextButton(
              onPressed: () => Navigator.pop(context, true),
              child: const Text('Discard'),
            ),
          ],
        ),
      );
      if (discard != true) return;
    }
    if (mounted) Navigator.pop(context, false);
  }

  @override
  void dispose() {
    name.dispose();
    description.dispose();
    body.dispose();
    super.dispose();
  }

  Future<void> readFile() async {
    final file = await openFile(
      acceptedTypeGroups: [
        const XTypeGroup(label: 'Skill', extensions: ['md']),
      ],
    );
    if (file == null || !mounted) return;
    try {
      final bytes = await file.readAsBytes();
      if (bytes.length > 8192) throw 'Choose a SKILL.md within 8 KiB.';
      final text = utf8.decode(bytes);
      body.text = text;
      // A suggestion only: the host parses and checks exact YAML metadata.
      final match = RegExp(
        r'^name:\s*["\x27]?([a-z0-9-]+)',
        multiLine: true,
      ).firstMatch(text);
      if (match != null) name.text = match.group(1)!;
      setState(() => error = null);
    } catch (_) {
      if (mounted) {
        setState(
          () => error =
              'Cannot import this file. Choose UTF-8 SKILL.md within 8 KiB.',
        );
      }
    }
  }

  Future<void> save() async {
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final text = widget.importing
          ? body.text
          : '---\nname: ${jsonEncode(name.text.trim())}\ndescription: ${jsonEncode(description.text.trim())}\n---\n${body.text}';
      await widget.bridge.call({
        'command': 'saveSkillText',
        'session': widget.session,
        'scope': widget.scope,
        'name': name.text.trim(),
        'text': text,
      });
      if (mounted) Navigator.pop(context, true);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => CloseOnEscape(
    onClose: close,
    child: PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) close();
      },
      child: AlertDialog(
        title: Text(widget.importing ? 'Import skill' : 'Create skill'),
        content: SizedBox(
          width: 560,
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (error != null) Text(error!),
                ...[
                  if (widget.importing)
                    TextButton.icon(
                      onPressed: pending ? null : readFile,
                      icon: const Icon(Icons.file_open_outlined),
                      label: const Text('Choose SKILL.md'),
                    ),
                  FieldLabel(
                    label: 'Name',
                    child: TextField(
                      key: const Key('create-skill-name'),
                      controller: name,
                      enabled: !pending,
                      decoration: const InputDecoration(
                        hintText: 'code-review',
                      ),
                    ),
                  ),
                  if (!widget.importing)
                    FieldLabel(
                      label: 'When to use it',
                      child: TextField(
                        key: const Key('create-skill-description'),
                        controller: description,
                        enabled: !pending,
                        decoration: const InputDecoration(),
                      ),
                    ),
                  const SizedBox(height: 12),
                  FieldLabel(
                    label: widget.importing ? 'SKILL.md' : 'Instructions',
                    child: TextField(
                      key: const Key('create-skill-text'),
                      controller: body,
                      enabled: !pending,
                      minLines: 5,
                      maxLines: 10,
                      decoration: InputDecoration(),
                    ),
                  ),
                ],
              ],
            ),
          ),
        ),
        actions: [
          TextButton(
            onPressed: pending ? null : close,
            child: const Text('Cancel'),
          ),
          FilledButton(
            key: const Key('create-skill-review'),
            onPressed: pending ? null : save,
            child: Text(
              pending
                  ? 'Saving…'
                  : widget.importing
                  ? 'Import skill'
                  : 'Save skill',
            ),
          ),
        ],
      ),
    ),
  );
}
