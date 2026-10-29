import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';
import 'settings_frame.dart';
import 'form_field_layout.dart';

class CodexCompatibility extends StatefulWidget {
  final ChatController chat;
  const CodexCompatibility({super.key, required this.chat});
  @override
  State<CodexCompatibility> createState() => _CodexCompatibilityState();
}

class _CodexCompatibilityState extends State<CodexCompatibility> {
  final home = TextEditingController();
  Map<String, dynamic>? report;
  Map<String, dynamic>? loaded;
  bool enabled = false, project = false, references = false, busy = false;
  bool dirty = false;
  String? error;
  String? setupStatus;
  SettingsDraft? draft;
  @override
  void initState() {
    super.initState();
    load();
    home.addListener(changed);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    draft = SettingsEmbedding.of(context)?.draft;
    draft?.save = save;
    draft?.isDirty = () => dirty;
  }

  @override
  void dispose() {
    home.dispose();
    super.dispose();
  }

  void changed() {
    dirty = true;
    if (mounted) setState(() {});
  }

  Future<void> load() async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final value = Map<String, dynamic>.from(
        await widget.chat.bridge.call({
          'command': 'codexSettings',
          'session': widget.chat.session,
        }) as Map,
      );
      final profile = value['profile'] as Map;
      home.text = (profile['home'] ?? value['detectedHome'] ?? '').toString();
      enabled = profile['enabled'] == true;
      project = profile['project'] == true;
      references = profile['referencedInstructions'] == true;
      loaded = value;
      report = Map<String, dynamic>.from(value['report'] as Map);
      dirty = false;
    } catch (e) {
      error = e.toString();
    }
    if (mounted) setState(() => busy = false);
  }

  Future<bool> save() async {
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final value = Map<String, dynamic>.from(
        await widget.chat.bridge.call({
          'command': 'saveCodexSettings',
          'session': widget.chat.session,
          'profile': {
            'enabled': enabled,
            'home': home.text.trim().isEmpty ? null : home.text.trim(),
            'project': project,
            'referencedInstructions': references,
          },
        }) as Map,
      );
      report = Map<String, dynamic>.from(value['report'] as Map);
      dirty = false;
      widget.chat.invalidateContext();
      if (mounted) setState(() => busy = false);
      return true;
    } catch (e) {
      if (mounted) {
        setState(() {
          error = e.toString();
          busy = false;
        });
      }
      return false;
    }
  }

  Future<void> openSetup() async {
    if (!await save() || !mounted) return;
    setState(() {
      busy = true;
      error = null;
      setupStatus = null;
    });
    try {
      await widget.chat.onOpenCodexSetupTerminal!();
      if (mounted) {
        setState(
          () => setupStatus = 'Setup terminal opened. Close Settings to use it, then return here to reload.',
        );
      }
    } catch (e) {
      if (mounted) setState(() => error = e.toString());
    }
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Codex setup',
    canClose: !busy,
    subtitle: 'Reuse a Codex profile’s MCP servers, command hooks, instructions and skills. Source files stay read-only. Model and login settings are not imported.',
    child: ListView(
      padding: const EdgeInsets.all(16),
      children: [
        SwitchListTile.adaptive(
          contentPadding: EdgeInsets.zero,
          title: const Text('Use this Codex setup'),
          value: enabled,
          onChanged: busy || widget.chat.busy
              ? null
              : (v) {
                  enabled = v;
                  changed();
                },
        ),
        const SizedBox(height: 12),
        FieldLabel(
          label: 'Codex profile folder',
          child: TextField(
            key: const Key('codex-profile-home'),
            controller: home,
            enabled: !busy && !widget.chat.busy,
            decoration: const InputDecoration(
              hintText: 'Absolute path to the Codex home folder',
            ),
          ),
        ),
        const SizedBox(height: 8),
        Wrap(
          spacing: 8,
          children: [
            TextButton(
              onPressed: busy
                  ? null
                  : () {
                      home.text = (loaded?['detectedHome'] ?? '').toString();
                    },
              child: const Text('Existing Codex profile'),
            ),
            if (loaded?['ownedHome'] != null)
              TextButton(
                onPressed: busy
                    ? null
                    : () {
                        home.text = loaded!['ownedHome'].toString();
                      },
                child: const Text('Separate setup folder'),
              ),
          ],
        ),
        SwitchListTile.adaptive(
          contentPadding: EdgeInsets.zero,
          title: const Text('Include project setup'),
          subtitle: const Text(
            'Load instructions, skills and .codex configuration from this project.',
          ),
          value: project,
          onChanged: busy || widget.chat.busy
              ? null
              : (v) {
                  project = v;
                  changed();
                },
        ),
        SwitchListTile.adaptive(
          contentPadding: EdgeInsets.zero,
          title: const Text('Load referenced instruction files'),
          subtitle: const Text(
            'Also read standalone @file lines, such as @RTK.md.',
          ),
          value: references,
          onChanged: busy || widget.chat.busy
              ? null
              : (v) {
                  references = v;
                  changed();
                },
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 8,
          children: [
            FilledButton(
              onPressed: busy || widget.chat.busy ? null : save,
              child: const Text('Save'),
            ),
            TextButton(
              onPressed: busy ? null : load,
              child: const Text('Reload'),
            ),
            if (widget.chat.onOpenCodexSetupTerminal != null)
              TextButton.icon(
                onPressed: busy || widget.chat.busy || home.text.trim().isEmpty
                    ? null
                    : openSetup,
                icon: const Icon(Icons.terminal, size: 18),
                label: const Text('Open setup terminal'),
              ),
          ],
        ),
        if (busy) const LinearProgressIndicator(),
        if (error != null)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 12),
            child: SelectableText(error!),
          ),
        if (setupStatus != null)
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 12),
            child: Text(setupStatus!),
          ),
        if (report != null) ...[
          const SizedBox(height: 16),
          Text(
            '${(report!['servers'] as List? ?? []).where((s) => s['enabled'] == true).length} configured servers · ${report!['hookCount'] ?? 0} command hooks',
          ),
          const SizedBox(height: 8),
          for (final item in report!['diagnostics'] as List? ?? [])
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 4),
              child: SelectableText(item.toString()),
            ),
          ExpansionTile(
            tilePadding: EdgeInsets.zero,
            childrenPadding: const EdgeInsets.all(12),
            title: const Text('Loaded sources'),
            children: [
              for (final item in report!['sources'] as List? ?? [])
                Align(
                  alignment: Alignment.centerLeft,
                  child: Padding(
                    padding: const EdgeInsets.only(bottom: 8),
                    child: SelectableText(item.toString()),
                  ),
                ),
            ],
          ),
          const SizedBox(height: 8),
          const Text(
            'Changes apply to the next task. Run third-party setup commands with CODEX_HOME set to the selected folder. Only installers that respect CODEX_HOME can use a separate setup folder.',
          ),
        ],
      ],
    ),
  );
}
