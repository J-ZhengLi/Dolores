import 'dart:convert';

import 'package:flutter/material.dart';

import 'bridge.dart';
import 'inspector.dart';
import 'settings_frame.dart';
import 'theme.dart';
import 'form_field_layout.dart';
import 'help_tip.dart';

class LifecycleHooksInspector extends StatefulWidget {
  final ChatBridge bridge;
  final String session;
  const LifecycleHooksInspector({
    super.key,
    required this.bridge,
    required this.session,
  });
  @override
  State<LifecycleHooksInspector> createState() => _HooksState();
}

class _HooksState extends State<LifecycleHooksInspector> {
  final name = TextEditingController(),
      script = TextEditingController(),
      args = TextEditingController();
  String event = 'beforeTool', mode = 'blocking', program = 'node';
  int seconds = 5, revision = 0;
  List<Map<String, dynamic>> hooks = [];
  Map? review;
  String? error, notice;
  bool busy = false;
  final scroll = ScrollController();
  int? editing;
  String baseline = '';
  String get draft => jsonEncode([
    hooks,
    name.text,
    script.text,
    args.text,
    event,
    mode,
    program,
    seconds,
  ]);
  bool get dirty =>
      baseline.isNotEmpty && (draft != baseline || review != null);
  Future<dynamic> call(Map<String, dynamic> request) => widget.bridge.call({
    'command': 'lifecycleHooks',
    'session': widget.session,
    'request': request,
  });
  @override
  void initState() {
    super.initState();
    act(load);
  }

  Future<void> act(Future<void> Function() action) async {
    if (busy) return;
    setState(() {
      busy = true;
      error = null;
      notice = null;
    });
    try {
      await action();
    } catch (e) {
      if (mounted) setState(() => error = e.toString());
    } finally {
      if (mounted) {
        setState(() => busy = false);
        if (error != null || notice != null) {
          WidgetsBinding.instance.addPostFrameCallback((_) {
            if (mounted && scroll.hasClients) scroll.jumpTo(0);
          });
        }
      }
    }
  }

  void changed() {
    setState(() => review = null);
  }

  void clear() {
    name.clear();
    script.clear();
    args.clear();
    editing = null;
  }

  void apply(Map result) {
    final configuration = result['configuration'] as Map;
    revision = configuration['revision'] as int;
    hooks = (configuration['hooks'] as List)
        .map((h) => Map<String, dynamic>.from(h['hook'] as Map))
        .toList();
    clear();
    review = null;
    baseline = draft;
  }

  Future<void> load() async {
    final result = await call({'action': 'view'});
    if (mounted) setState(() => apply(result as Map));
  }

  void stage() {
    if (name.text.trim().isEmpty || script.text.trim().isEmpty) {
      setState(() => error = 'Enter a name and a project script.');
      return;
    }
    final hook = <String, dynamic>{
      'name': name.text.trim(),
      'event': event,
      'mode': mode,
      'program': program,
      'script': script.text.trim(),
      'args': args.text.isEmpty ? <String>[] : args.text.split('\n'),
      'timeoutSeconds': seconds,
    };
    setState(() {
      if (editing == null) {
        hooks.add(hook);
      } else {
        hooks[editing!] = hook;
      }
      clear();
      review = null;
      error = null;
    });
  }

  Future<void> inspect() => act(() async {
    if (name.text.isNotEmpty ||
        script.text.isNotEmpty ||
        args.text.isNotEmpty) {
      throw 'Use Add to review or Update entry before reviewing the list.';
    }
    final result = await call({
      'action': 'review',
      'revision': revision,
      'hooks': hooks,
    });
    if (mounted) setState(() => review = result as Map);
  });
  Future<bool> save() async {
    if (review == null) {
      setState(() => error = 'Review the hook list before enabling.');
      return false;
    }
    var saved = false;
    await act(() async {
      final result = await call({
        'action': 'enable',
        'token': review!['token'],
      });
      if (mounted) {
        setState(() {
          apply(result as Map);
          notice = 'Hooks enabled for the next foreground task.';
          saved = true;
        });
      }
    });
    return saved;
  }

  Widget choice(
    String key,
    String label,
    String value,
    Map<String, String> values,
    void Function(String) change,
  ) => FieldLabel(
    label: label,
    child: DropdownButtonFormField<String>(
      key: ValueKey('$key-$value'),
      initialValue: value,
      isExpanded: true,
      items: values.entries
          .map((e) => DropdownMenuItem(value: e.key, child: Text(e.value)))
          .toList(),
      onChanged: busy
          ? null
          : (v) {
              change(v!);
              changed();
            },
    ),
  );
  Widget text(
    String key,
    String label,
    TextEditingController controller, {
    String? hint,
    int lines = 1,
  }) => Padding(
    padding: const EdgeInsets.only(top: 12),
    child: FieldLabel(
      label: label,
      child: TextField(
        key: Key(key),
        controller: controller,
        enabled: !busy,
        minLines: lines,
        maxLines: lines == 1 ? 1 : 4,
        onChanged: (_) => changed(),
        decoration: InputDecoration(hintText: hint),
      ),
    ),
  );
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    reportSettingsDraft(context, dirty: () => dirty, save: save);
    return InspectorFrame(
      title: 'Lifecycle hooks',
      subtitle: 'This working folder',
      canClose: !busy,
      child: Column(
        children: [
          if (busy) const LinearProgressIndicator(minHeight: 2),
          if (error != null)
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
              child: SelectableText(
                error!,
                maxLines: 4,
                style: TextStyle(color: p.errorText),
              ),
            ),
          if (notice != null)
            Padding(
              padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
              child: Text(notice!),
            ),
          Expanded(
            child: ListView(
              key: const Key('hooks-scroll'),
              controller: scroll,
              padding: const EdgeInsets.all(20),
              children: [
                const Text('Run reviewed project scripts at task boundaries.'),
                const SizedBox(height: 8),
                const HelpLabel(
                  help: 'Scripts run with your account permissions and can change files or use the network. Only the top-level script is pinned. Changes require review; opening Settings never runs a hook. Hook output stays in local run evidence and is not sent to the model.',
                  child: Text('Review scripts before enabling.'),
                ),
                const SizedBox(height: 12),
                for (var i = 0; i < hooks.length; i++)
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(hooks[i]['name'] as String),
                    subtitle: Text(
                      '${hooks[i]['event']} · ${hooks[i]['mode']}\n${hooks[i]['program']} · ${hooks[i]['script']}',
                    ),
                    onTap: busy
                        ? null
                        : () {
                            if (name.text.isNotEmpty ||
                                script.text.isNotEmpty ||
                                args.text.isNotEmpty) {
                              setState(
                                () => error = 'Add or update the current entry before editing another hook.',
                              );
                              return;
                            }
                            setState(() {
                              editing = i;
                              final h = hooks[i];
                              name.text = h['name'];
                              script.text = h['script'];
                              args.text = (h['args'] as List).join('\n');
                              event = h['event'];
                              mode = h['mode'];
                              program = h['program'];
                              seconds = h['timeoutSeconds'];
                              review = null;
                            });
                          },
                    trailing: IconButton(
                      tooltip: 'Remove ${hooks[i]['name']}',
                      onPressed: busy
                          ? null
                          : () => setState(() {
                              hooks.removeAt(i);
                              clear();
                              review = null;
                            }),
                      icon: const Icon(Icons.remove_circle_outline),
                    ),
                  ),
                if (hooks.length < 4 || editing != null) ...[
                  text('hook-name', 'Name', name, hint: 'project-check'),
                  const SizedBox(height: 12),
                  choice(
                    'hook-event',
                    'When',
                    event,
                    const {
                      'taskStart': 'Task start',
                      'beforeTool': 'Before a tool',
                      'afterTool': 'After a tool',
                      'beforeCompaction': 'Before compaction',
                      'taskComplete': 'After a saved turn',
                    },
                    (v) {
                      event = v;
                      if (['afterTool', 'taskComplete'].contains(v)) {
                        mode = 'observe';
                      }
                    },
                  ),
                  const SizedBox(height: 12),
                  choice('hook-mode', 'Behavior', mode, {
                    'observe': 'Observe',
                    if (!['afterTool', 'taskComplete'].contains(event))
                      'blocking': 'Require a passing check',
                  }, (v) => mode = v),
                  const SizedBox(height: 12),
                  choice('hook-program', 'Runtime', program, const {
                    'node': 'Node',
                    'python': 'Python',
                  }, (v) => program = v),
                  text(
                    'hook-script',
                    'Project script',
                    script,
                    hint: 'scripts/check.cjs',
                  ),
                  text('hook-args', 'Arguments (one per line)', args, lines: 2),
                  const SizedBox(height: 12),
                  choice('hook-seconds', 'Time limit', seconds.toString(), {
                    for (var i = 1; i <= 10; i++) '$i': '$i seconds',
                  }, (v) => seconds = int.parse(v)),
                  Align(
                    alignment: Alignment.centerLeft,
                    child: TextButton(
                      key: const Key('hook-stage'),
                      onPressed: busy ? null : stage,
                      child: Text(
                        editing == null ? 'Add to review' : 'Update entry',
                      ),
                    ),
                  ),
                ],
                if (review != null) ...[
                  const Divider(),
                  const Text('Review automatic execution'),
                  const Text(
                    'Enable authorizes these scripts for the listed events in this folder. Their effects may remain after Stop or failure.',
                  ),
                  SelectableText('Folder: ${review!['root']}'),
                  for (final saved in review!['configuration']['hooks'] as List)
                    ExpansionTile(
                      title: Text(saved['hook']['name']),
                      children: [
                        SelectableText(
                          const JsonEncoder.withIndent('  ').convert(saved),
                        ),
                      ],
                    ),
                  for (final source in review!['scripts'] as List? ?? [])
                    ExpansionTile(
                      title: Text('Script · ${source['name']}'),
                      children: [
                        SelectableText(source['text'] as String),
                        if ((source['omittedBytes'] as int) > 0)
                          Text(
                            '${source['omittedBytes']} bytes omitted. Inspect the complete project file before enabling.',
                          ),
                      ],
                    ),
                ],
              ],
            ),
          ),
          const Divider(height: 1),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                TextButton(
                  key: const Key('hooks-refresh'),
                  onPressed: busy
                      ? null
                      : () async {
                          if (dirty &&
                              !await resolveSettingsDraft(
                                context,
                                save: save,
                              )) {
                            return;
                          }
                          await act(load);
                        },
                  child: const Text('Refresh'),
                ),
                TextButton(
                  key: const Key('hooks-disable'),
                  onPressed: busy
                      ? null
                      : () => act(() async {
                          final result = await call({
                            'action': 'disable',
                            'revision': revision,
                          });
                          if (mounted) {
                            setState(() {
                              apply(result as Map);
                              notice = 'All hooks disabled.';
                            });
                          }
                        }),
                  child: const Text('Disable all'),
                ),
                if (review == null)
                  FilledButton(
                    key: const Key('hooks-review'),
                    onPressed: busy ? null : inspect,
                    child: const Text('Review hooks'),
                  ),
                if (review != null)
                  FilledButton(
                    key: const Key('hooks-enable'),
                    onPressed: busy ? null : save,
                    child: const Text('Enable reviewed hooks'),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    scroll.dispose();
    name.dispose();
    script.dispose();
    args.dispose();
    super.dispose();
  }
}
