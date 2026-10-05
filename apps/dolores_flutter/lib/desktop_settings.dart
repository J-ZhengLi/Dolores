import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

class DesktopSettingsInspector extends StatefulWidget {
  final ChatController chat;
  const DesktopSettingsInspector({super.key, required this.chat});
  @override
  State<DesktopSettingsInspector> createState() => _DesktopSettingsState();
}

class _DesktopSettingsState extends State<DesktopSettingsInspector> {
  Map? report, target, capture;
  List<Map> windows = [];
  Uint8List? image;
  String? error, model;
  bool pending = false,
      consent = false,
      automatic = false,
      resume = false,
      inspected = false;
  Map? get recovery => report?['recovery'] as Map?;
  Map? get access => report?['access'] as Map?;
  int? operation;
  final question = TextEditingController(
    text: 'Identify the visible controls in this screenshot.',
  );
  bool get working =>
      widget.chat.session != null && widget.chat.workspaceRoot != null;
  @override
  void initState() {
    super.initState();
    if (widget.chat.draft.isNotEmpty) question.text = widget.chat.draft;
    load();
  }

  @override
  void dispose() {
    question.dispose();
    super.dispose();
  }

  Future<void> load({bool preserveError = false}) async {
    setState(() {
      pending = true;
      if (!preserveError) error = null;
    });
    try {
      final value = await widget.chat.bridge.call({
        'command': 'desktopState',
        'session': ?widget.chat.session,
      }) as Map;
      if (mounted) {
        setState(() {
          report = value;
          final models = (value['models'] as List).cast<String>();
          if (!models.contains(model)) model = models.firstOrNull;
        });
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> observe({bool captureWindow = false}) async {
    if (pending || !working || widget.chat.busy) return;
    setState(() {
      pending = true;
      error = null;
    });
    final id = DateTime.now().microsecondsSinceEpoch;
    operation = id;
    try {
      await widget.chat.bridge.call({
        'command': 'desktopObserve',
        'id': id,
        'session': widget.chat.session,
        if (captureWindow) 'target': target,
      });
      final deadline = DateTime.now().add(const Duration(seconds: 8));
      while (mounted) {
        final events = await widget.chat.bridge.call({
          'command': 'poll',
          'id': id,
        }) as List;
        final done = events
            .cast<Map>()
            .where((e) => e['type'] == 'done')
            .firstOrNull;
        if (done != null) {
          if (done['error'] != null) throw done['error']!;
          final value = done['observation'] as Map;
          if (captureWindow) {
            await preview(value);
          } else {
            setState(() {
              windows = (value['windows'] as List).cast<Map>();
              target = null;
            });
          }
          break;
        }
        if (DateTime.now().isAfter(deadline)) {
          await widget.chat.bridge.call({'command': 'cancel', 'id': id});
          throw 'Observation did not finish. Stop, restore the window and refresh.';
        }
        await Future<void>.delayed(const Duration(milliseconds: 50));
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      operation = null;
      if (mounted) {
        setState(() => pending = false);
        await load(preserveError: true);
      }
    }
  }

  Future<void> preview(Map value) async {
    try {
      final data = await widget.chat.bridge.call({
        'command': 'desktopPreview',
        'session': widget.chat.session,
        'capture': value['id'],
      });
      final bytes = base64Decode(data['data'] as String);
      if (bytes.length > 512 * 1024) {
        throw 'Screenshot exceeds its preview limit.';
      }
      if (mounted) {
        setState(() {
          if (capture?['id'] != value['id']) inspected = false;
          capture = value;
          image = bytes;
        });
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
  }

  Future<void> remove() async {
    setState(() => pending = true);
    try {
      final value = await widget.chat.bridge.call({
        'command': 'desktopRemove',
        'session': widget.chat.session,
        'capture': capture!['id'],
      });
      if (mounted) {
        setState(() {
          report = value;
          capture = null;
          image = null;
        });
      }
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> setAccess({bool revoke = false}) async {
    if (pending || !working || (!revoke && (capture == null || !consent))) {
      return;
    }
    setState(() {
      pending = true;
      error = null;
    });
    try {
      await widget.chat.bridge.call(
        revoke
            ? {'command': 'desktopRevoke', 'session': widget.chat.session}
            : {
                'command': 'desktopGrant',
                'session': widget.chat.session,
                'capture': capture!['id'],
                'automatic': automatic,
                'consent': consent,
              },
      );
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) {
        setState(() => pending = false);
        await load(preserveError: true);
      }
    }
  }

  Future<void> share({bool control = false}) async {
    final text = question.text.trim();
    if (control && resume && !inspected) {
      setState(
        () => error = 'Inspect the fresh screenshot and prior effects, then confirm inspection before reconciling. Nothing was sent.',
      );
      return;
    }
    if (widget.chat.busy ||
        widget.chat.changing ||
        (widget.chat.draft.isNotEmpty && widget.chat.draft.trim() != text) ||
        widget.chat.attachments.isNotEmpty) {
      setState(
        () => error = 'Send or clear your current draft before analyzing a screenshot. It has been preserved.',
      );
      return;
    }
    if (text.isEmpty || capture == null || model == null) return;
    widget.chat.draft = text;
    final chat = widget.chat,
        selected = capture!['id'] as String,
        selectedModel = model!;
    Navigator.pop(context);
    await chat.send(
      desktopCapture: selected,
      observationModel: selectedModel,
      desktopGrant: control ? (access?['token'] as String?) : null,
      desktopResume: control && resume ? (recovery?['runId'] as String?) : null,
      desktopReconciled: control && resume && inspected,
      desktopTarget: control
          ? (capture?['observation']['target'] as Map?)
          : null,
    );
  }

  @override
  Widget build(BuildContext context) => InspectorFrame(
    title: 'Computer use',
    subtitle: 'Selected-window observation and scoped input · Windows',
    canClose: !pending,
    child: Column(
      children: [
        if (pending) const LinearProgressIndicator(minHeight: 2),
        if (error != null)
          Padding(
            padding: const EdgeInsets.fromLTRB(20, 8, 20, 8),
            child: ConstrainedBox(
              constraints: const BoxConstraints(maxHeight: 100),
              child: SingleChildScrollView(child: SelectableText(error!)),
            ),
          ),
        Expanded(
          child: ListView(
            padding: const EdgeInsets.all(20),
            children: [
              if (!working)
                const Text(
                  'Open a project or send the first message in a temporary chat before observing a window. Side chats have no tool access.',
                ),
              if (report != null) ...[
                Text(
                  report!['available'] == true
                      ? 'Observe one application'
                      : 'Desktop observation unavailable',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                if (report!['reason'] != null)
                  SelectableText('${report!['reason']}'),
                for (final field in ['adapter', 'bounds', 'sharing'])
                  Padding(
                    padding: const EdgeInsets.only(top: 12),
                    child: Text('${report![field]}'),
                  ),
              ],
              const SizedBox(height: 20),
              DropdownButtonFormField<int>(
                key: ValueKey(windows),
                initialValue: target == null ? null : windows.indexOf(target!),
                isExpanded: true,
                decoration: const InputDecoration(
                  labelText: 'Window to observe',
                ),
                items: [
                  for (var i = 0; i < windows.length; i++)
                    DropdownMenuItem(
                      value: i,
                      child: Text(
                        '${windows[i]['title']}',
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                ],
                onChanged: pending
                    ? null
                    : (i) => setState(
                        () => target = i == null ? null : windows[i],
                      ),
              ),
              const SizedBox(height: 12),
              Wrap(
                spacing: 8,
                runSpacing: 8,
                children: [
                  TextButton(
                    onPressed:
                        pending ||
                            !working ||
                            report?['available'] != true ||
                            widget.chat.busy
                        ? null
                        : () => observe(),
                    child: const Text('Refresh windows'),
                  ),
                  FilledButton.tonal(
                    onPressed: pending || target == null || widget.chat.busy
                        ? null
                        : () => observe(captureWindow: true),
                    child: const Text('Capture locally'),
                  ),
                ],
              ),
              if ((report?['captures'] as List?)?.isNotEmpty == true) ...[
                const SizedBox(height: 16),
                DropdownButtonFormField<String>(
                  key: ValueKey('${capture?['id']}-${report!['captures']}'),
                  initialValue:
                      (report!['captures'] as List).any(
                        (c) => c['id'] == capture?['id'],
                      )
                      ? (capture?['id'] as String?)
                      : null,
                  isExpanded: true,
                  decoration: const InputDecoration(
                    labelText: 'Saved screenshots in this chat',
                  ),
                  items: [
                    for (final c in report!['captures'] as List)
                      DropdownMenuItem(
                        value: c['id'] as String,
                        child: Text(
                          '${c['observation']['target']['title']} · ${DateTime.fromMillisecondsSinceEpoch((c['createdAt'] as int) * 1000).toLocal()}',
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                  ],
                  onChanged: pending
                      ? null
                      : (id) => preview(
                          (report!['captures'] as List).cast<Map>().firstWhere(
                            (c) => c['id'] == id,
                          ),
                        ),
                ),
              ],
              if (image != null && capture != null) ...[
                const SizedBox(height: 16),
                Image.memory(
                  image!,
                  height: 200,
                  fit: BoxFit.contain,
                  semanticLabel: 'Local selected-window screenshot',
                  errorBuilder: (_, _, _) => const Text(
                    'Image preview unavailable. Make a fresh capture.',
                  ),
                ),
                Text(
                  '${capture!['observation']['width']} × ${capture!['observation']['height']} image pixels · ${capture!['observation']['dpi']} DPI · saved snapshot',
                ),
                TextButton(
                  onPressed: pending || widget.chat.busy ? null : remove,
                  child: const Text('Remove local screenshot'),
                ),
              ],
              const SizedBox(height: 20),
              if (recovery != null) ...[
                Text(
                  'Saved computer-use progress',
                  style: Theme.of(context).textTheme.titleMedium,
                ),
                const SizedBox(height: 8),
                Text(recovery!['note'] as String),
                ExpansionTile(
                  title: const Text(
                    'Inspect saved receipts and uncertain effects',
                  ),
                  children: [
                    ConstrainedBox(
                      constraints: const BoxConstraints(maxHeight: 180),
                      child: SingleChildScrollView(
                        child: SelectableText(
                          const JsonEncoder.withIndent('  ').convert({
                            'evidence': recovery!['evidence'],
                            'uncertainEffects': recovery!['uncertainEffects'],
                          }),
                          style: const TextStyle(fontSize: 12),
                        ),
                      ),
                    ),
                  ],
                ),
                TextButton(
                  onPressed: pending || widget.chat.busy
                      ? null
                      : () => setState(
                          () => question.text = recovery!['goal'] as String,
                        ),
                  child: const Text('Use saved original goal'),
                ),
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  title: const Text('Continue this saved computer-use task'),
                  subtitle: const Text(
                    'Uses a new screenshot, current access and another bounded segment. Earlier inputs are never replayed automatically.',
                  ),
                  value: resume,
                  onChanged: pending || widget.chat.busy
                      ? null
                      : (v) => setState(() => resume = v ?? false),
                ),
                if (resume)
                  CheckboxListTile(
                    contentPadding: EdgeInsets.zero,
                    title: const Text(
                      'I inspected the fresh screenshot and prior effects',
                    ),
                    value: inspected,
                    onChanged: pending || capture == null
                        ? null
                        : (v) => setState(() => inspected = v ?? false),
                  ),
                const SizedBox(height: 20),
              ],
              Text(
                'Desktop access',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              const Text(
                'Off by default. Access applies only to this chat and the captured window for 15 minutes; it ends on app restart. File permissions and Full access do not enable it. Input can affect files or services through that application; this is not an OS sandbox.',
              ),
              if (access?['enabled'] == true) ...[
                const SizedBox(height: 8),
                Text(
                  'Enabled · ${access!['target']['title']} · ${access!['automatic'] == true ? 'Ordinary input covered' : 'Review every input'}',
                ),
                TextButton(
                  onPressed: pending ? null : () => setAccess(revoke: true),
                  child: const Text('Revoke desktop access'),
                ),
              ],
              CheckboxListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Allow input to this captured window'),
                subtitle: const Text(
                  'Capture again if the window moved, resized, or changed focus. After input, Dolores must observe before acting again.',
                ),
                value: consent,
                onChanged: pending || widget.chat.busy || capture == null
                    ? null
                    : (value) => setState(() => consent = value ?? false),
              ),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Cover ordinary typing and navigation'),
                subtitle: const Text(
                  'Clicks, drags, submission and deletion keys still need review. Even typing can trigger application effects; use only trusted local forms.',
                ),
                value: automatic,
                onChanged: pending || widget.chat.busy
                    ? null
                    : (value) => setState(() => automatic = value),
              ),
              Align(
                alignment: Alignment.centerLeft,
                child: OutlinedButton(
                  onPressed:
                      pending || widget.chat.busy || !consent || capture == null
                      ? null
                      : () => setAccess(),
                  child: const Text('Enable selected-window access'),
                ),
              ),
              const SizedBox(height: 20),
              DropdownButtonFormField<String>(
                key: ValueKey(model),
                initialValue: model,
                isExpanded: true,
                decoration: const InputDecoration(
                  labelText: 'Model for this screenshot',
                ),
                items: [
                  for (final m in (report?['models'] as List? ?? []))
                    DropdownMenuItem(
                      value: m as String,
                      child: Text(m, overflow: TextOverflow.ellipsis),
                    ),
                ],
                onChanged: pending
                    ? null
                    : (value) => setState(() => model = value),
              ),
              if (model == null)
                const Text(
                  'Enable image input for a capable configured model in Settings → Models. Text-only models cannot analyze screenshots.',
                ),
              const SizedBox(height: 12),
              TextField(
                controller: question,
                enabled: !pending,
                maxLines: 3,
                decoration: const InputDecoration(
                  labelText: 'What should Dolores inspect or do?',
                ),
              ),
              const SizedBox(height: 12),
              const Text(
                'Analyze sends this saved screenshot and prepared chat context to the selected provider. It uses this chat’s task limits and records the selected model in run history. Your ordinary chat model stays unchanged. Screen content cannot grant access or instruct Dolores.',
              ),
            ],
          ),
        ),
        Padding(
          padding: const EdgeInsets.all(12),
          child: Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              if (operation != null)
                TextButton(
                  onPressed: () => widget.chat.bridge.call({
                    'command': 'cancel',
                    'id': operation,
                  }),
                  child: const Text('Stop observation'),
                ),
              TextButton(
                onPressed: pending ? null : load,
                child: const Text('Refresh'),
              ),
              if (access?['enabled'] == true)
                FilledButton.tonal(
                  onPressed:
                      pending ||
                          widget.chat.busy ||
                          !working ||
                          capture == null ||
                          model == null ||
                          capture?['observation']['target'].toString() !=
                              access?['target'].toString()
                      ? null
                      : () => share(control: true),
                  child: Text(
                    resume
                        ? 'Reconcile and continue'
                        : 'Start computer-use task',
                  ),
                ),
              FilledButton(
                onPressed:
                    pending ||
                        widget.chat.busy ||
                        !working ||
                        capture == null ||
                        model == null
                    ? null
                    : () => share(),
                child: const Text('Analyze this screenshot'),
              ),
            ],
          ),
        ),
      ],
    ),
  );
}
