import 'dart:convert';
import 'dart:async';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';

import 'bridge.dart';

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
  Map<String, dynamic>? review;
  Future<void> close() async {
    if (pending) return;
    if (name.text.isNotEmpty ||
        description.text.isNotEmpty ||
        body.text.isNotEmpty) {
      final discard = await showDialog<bool>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Discard skill draft?'),
          content: const Text('Your skill has not been activated.'),
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
    final token = review?['token'];
    if (token != null) {
      unawaited(
        widget.bridge
            .call({'command': 'cancelSkillReview', 'token': token})
            .catchError((_) => null),
      );
    }
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

  Future<void> inspect() async {
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final text = widget.importing
          ? body.text
          : '---\nname: ${jsonEncode(name.text.trim())}\ndescription: ${jsonEncode(description.text.trim())}\n---\n${body.text}';
      final value = await widget.bridge.call({
        'command': 'reviewSkillText',
        'session': widget.session,
        'scope': widget.scope,
        'name': name.text.trim(),
        'text': text,
      }) as Map;
      if (mounted) setState(() => review = value.cast<String, dynamic>());
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> activateSkill() async {
    setState(() {
      pending = true;
      error = null;
    });
    try {
      await widget.bridge.call({
        'command': 'activateSkill',
        'session': widget.session,
        'token': review!['token'],
      });
      if (mounted) Navigator.pop(context, true);
    } catch (e) {
      if (mounted) {
        setState(() {
          error = '$e';
          review = null;
        });
      }
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: false,
    onPopInvokedWithResult: (didPop, _) {
      if (!didPop) close();
    },
    child: AlertDialog(
      title: Text(
        review != null
            ? 'Review skill'
            : widget.importing
            ? 'Import skill'
            : 'Create skill',
      ),
      content: SizedBox(
        width: 560,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              if (error != null) Text(error!),
              if (review == null) ...[
                if (widget.importing)
                  TextButton.icon(
                    onPressed: pending ? null : readFile,
                    icon: const Icon(Icons.file_open_outlined),
                    label: const Text('Choose SKILL.md'),
                  ),
                TextField(
                  key: const Key('create-skill-name'),
                  controller: name,
                  enabled: !pending,
                  decoration: const InputDecoration(
                    labelText: 'Name',
                    hintText: 'code-review',
                  ),
                ),
                if (!widget.importing)
                  TextField(
                    key: const Key('create-skill-description'),
                    controller: description,
                    enabled: !pending,
                    decoration: const InputDecoration(
                      labelText: 'When to use it',
                    ),
                  ),
                const SizedBox(height: 12),
                TextField(
                  key: const Key('create-skill-text'),
                  controller: body,
                  enabled: !pending,
                  minLines: 5,
                  maxLines: 10,
                  decoration: InputDecoration(
                    labelText: widget.importing ? 'SKILL.md' : 'Instructions',
                  ),
                ),
              ] else ...[
                Text(
                  'Use ${widget.scope == 'global' ? 'in all projects' : 'in this project'}. Activation shares these instructions with your model. Tools keep their existing access rules.',
                ),
                const SizedBox(height: 12),
                SelectableText(review!['document']['text'] as String),
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
        if (review != null)
          TextButton(
            onPressed: pending ? null : () => setState(() => review = null),
            child: const Text('Edit'),
          ),
        FilledButton(
          key: const Key('create-skill-review'),
          onPressed: pending
              ? null
              : review == null
              ? inspect
              : activateSkill,
          child: Text(
            pending
                ? 'Working…'
                : review == null
                ? 'Review'
                : 'Activate skill',
          ),
        ),
      ],
    ),
  );
}
