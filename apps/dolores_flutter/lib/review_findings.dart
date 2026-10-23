import 'package:flutter/material.dart';

import 'git_host.dart';
import 'inspector.dart';

Widget _reviewBody(Widget child) =>
    Padding(padding: const EdgeInsets.all(16), child: child);

String reviewId() =>
    'review-${DateTime.now().microsecondsSinceEpoch}-${_sequence++}';
int _sequence = 0;

Future<void> showFindingEditor(
  BuildContext context,
  GitHost git,
  GitWorkspace w,
  Map diff,
  String side,
  int line,
) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => FindingEditor(
    git: git,
    workspace: w,
    diff: Map<String, dynamic>.from(diff),
    side: side,
    line: line,
  ),
);

class FindingEditor extends StatefulWidget {
  final GitHost git;
  final GitWorkspace workspace;
  final Map<String, dynamic> diff;
  final String side;
  final int line;
  const FindingEditor({
    super.key,
    required this.git,
    required this.workspace,
    required this.diff,
    required this.side,
    required this.line,
  });
  @override
  State<FindingEditor> createState() => _FindingEditorState();
}

class _FindingEditorState extends State<FindingEditor> {
  final note = TextEditingController();
  final id = reviewId();
  late final session = widget.workspace.session;
  bool saving = false;
  String? error;
  @override
  void dispose() {
    note.dispose();
    super.dispose();
  }

  Future<void> save() async {
    if (saving || note.text.trim().isEmpty) return;
    setState(() => saving = true);
    final w = widget.workspace;
    try {
      if (w.session != session) {
        throw StateError(
          'The selected chat changed. Keep this note and return to its original comparison.',
        );
      }
      final state = widget.git.reviewState(w);
      final ok = state.pending != null
          ? await widget.git.checkFinding(w)
          : await widget.git.runFinding(w, {
              'action': 'capture',
              'id': id,
              'comparison': widget.git.comparisonFor(w, widget.diff),
              'cursor': widget.diff['cursor'] ?? 0,
              'sideBySide': widget.diff['sideBySide'] ?? w.sideBySide,
              'side': widget.side,
              'line': widget.line,
              'text': note.text,
            });
      if (!mounted) return;
      if (ok) {
        Navigator.pop(context);
        return;
      }
      error =
          state.error ??
          'Finding was not saved. Keep the note and inspect status.';
    } catch (e) {
      error = '$e';
    }
    if (mounted) setState(() => saving = false);
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: widget.git,
    builder: (context, _) {
      final state = widget.git.reviewState(widget.workspace, session);
      final pending = state.pending != null;
      return PopScope(
        canPop: !saving && !pending,
        child: InspectorFrame(
          title: 'Comment on ${widget.side} line ${widget.line}',
          subtitle: 'Save feedback at this exact comparison',
          onClose: () => Navigator.pop(context),
          canClose: !saving && !pending,
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(widget.diff['path'] as String),
                const SizedBox(height: 12),
                TextField(
                  key: const Key('review-note'),
                  controller: note,
                  minLines: 3,
                  maxLines: 8,
                  readOnly: saving || pending,
                  decoration: const InputDecoration(
                    labelText: 'Finding',
                    hintText: 'Describe the problem and expected result',
                  ),
                ),
                if (error != null)
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 12),
                    child: SelectableText(error!),
                  ),
                const SizedBox(height: 12),
                Wrap(
                  spacing: 8,
                  children: [
                    FilledButton(
                      onPressed:
                          saving ||
                              state.loading ||
                              (!pending && widget.workspace.busy)
                          ? null
                          : save,
                      child: Text(pending ? 'Check status' : 'Save finding'),
                    ),
                    if (pending)
                      TextButton(
                        onPressed: saving
                            ? null
                            : () => widget.git.stop(widget.workspace),
                        child: const Text('Stop'),
                      ),
                  ],
                ),
                const SizedBox(height: 8),
                const Text('Saving a finding does not send it to the agent.'),
              ],
            ),
          ),
        ),
      );
    },
  );
}

Future<void> showReviewFindings(
  BuildContext context,
  GitHost git,
  GitWorkspace w,
) => showDialog<void>(
  context: context,
  barrierDismissible: false,
  builder: (_) => ReviewFindingsInspector(git: git, workspace: w),
);

class ReviewFindingsInspector extends StatefulWidget {
  final GitHost git;
  final GitWorkspace workspace;
  const ReviewFindingsInspector({
    super.key,
    required this.git,
    required this.workspace,
  });
  @override
  State<ReviewFindingsInspector> createState() =>
      _ReviewFindingsInspectorState();
}

class _ReviewFindingsInspectorState extends State<ReviewFindingsInspector> {
  GitWorkspace get w => widget.workspace;
  GitHost get git => widget.git;
  late final session = w.session;
  final resolution = TextEditingController();
  String? submissionId, submissionKey;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) git.loadFindings(w);
    });
  }

  @override
  void dispose() {
    resolution.dispose();
    super.dispose();
  }

  Future<void> submit(bool steer) async {
    final state = git.reviewState(w, session);
    final selected = [
      for (final id in state.selected)
        if (state.items[id] != null)
          {'id': id, 'revision': state.items[id]!['revision']},
    ];
    final key = '$steer:$selected';
    if (submissionKey != key) {
      submissionKey = key;
      submissionId = reviewId();
    }
    await git.runFinding(w, {
      'action': 'submit',
      'submission': {
        'id': submissionId,
        'mode': steer ? 'steer' : 'queue',
        'selected': selected,
      },
    });
  }

  Future<void> resolve(Map f, Map diff) async {
    resolution.clear();
    final note = await showDialog<String>(
      context: context,
      builder: (c) => AlertDialog(
        title: const Text('Resolve finding'),
        content: TextField(
          key: const Key('resolution-note'),
          controller: resolution,
          minLines: 2,
          maxLines: 5,
          decoration: const InputDecoration(labelText: 'What did you verify?'),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(c),
            child: const Text('Keep open'),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(c, resolution.text),
            child: const Text('Save resolution'),
          ),
        ],
      ),
    );
    if (note == null || note.trim().isEmpty) return;
    await git.runFinding(w, {
      'action': 'resolve',
      'id': f['id'],
      'revision': f['revision'],
      'comparison': git.comparisonFor(w, diff),
      'note': note,
    });
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: git,
    builder: (context, _) {
      final state = git.reviewState(w, session);
      final diff = w.tabs[w.activeTab];
      final locked = state.loading || w.busy || w.session != session;
      return PopScope(
        canPop: !state.loading && state.pending == null,
        child: InspectorFrame(
          title: 'Review findings',
          subtitle: 'Saved notes and repair delivery',
          onClose: () => Navigator.pop(context),
          canClose: !state.loading && state.pending == null,
          child: _reviewBody(
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Choose feedback to send. Delivery and verified resolution are tracked separately.',
                ),
                if (state.error != null)
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    child: SelectableText(state.error!),
                  ),
                if (state.notice != null)
                  Padding(
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    child: Text(state.notice!),
                  ),
                if (state.pending != null)
                  Wrap(
                    spacing: 8,
                    children: [
                      TextButton(
                        onPressed: state.loading
                            ? null
                            : () => git.checkFinding(w),
                        child: const Text('Check status'),
                      ),
                      TextButton(
                        onPressed: state.loading ? null : () => git.stop(w),
                        child: const Text('Stop'),
                      ),
                    ],
                  ),
                Expanded(
                  child: ListView(
                    children: [
                      if (state.items.isEmpty && !state.loading)
                        const Padding(
                          padding: EdgeInsets.all(16),
                          child: Text(
                            'No findings yet. Use a line comment in a diff.',
                          ),
                        ),
                      for (final f in state.items.values)
                        Builder(
                          builder: (context) {
                            final anchor = f['anchor'] as Map;
                            final basis = anchor['comparison'] as Map;
                            final current = git.findingCurrent(w, f, diff);
                            final resolved = f['resolution'] != null;
                            final eligible =
                                diff != null &&
                                basis['path'] == diff['path'] &&
                                basis['revision'] != diff['revision'] &&
                                basis['digest'] != diff['digest'] &&
                                diff['revision'] == w.status?['revision'];
                            return Card(
                              child: Padding(
                                padding: const EdgeInsets.all(12),
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    CheckboxListTile(
                                      contentPadding: EdgeInsets.zero,
                                      value: state.selected.contains(f['id']),
                                      onChanged:
                                          locked ||
                                              resolved ||
                                              (!current &&
                                                  !state.selected.contains(
                                                    f['id'],
                                                  ))
                                          ? null
                                          : (v) {
                                              if (v == true) {
                                                state.selected.add(
                                                  f['id'] as String,
                                                );
                                              } else {
                                                state.selected.remove(f['id']);
                                              }
                                              git.changed();
                                            },
                                      title: Text(
                                        f['text'] as String,
                                        style: TextStyle(
                                          color: Theme.of(context)
                                              .colorScheme
                                              .onSurface,
                                        ),
                                      ),
                                      subtitle: Text(
                                        '${basis['path']} · ${anchor['side']} line ${anchor['line']} · ${resolved
                                            ? 'Resolved'
                                            : !current
                                            ? 'Location changed or not open'
                                            : (f['submissions'] as List).isEmpty
                                            ? 'Open'
                                            : 'Submitted'}',
                                      ),
                                    ),
                                    SelectableText(
                                      anchor['excerpt'] as String,
                                      style: const TextStyle(
                                        fontFamily: 'Consolas',
                                        fontSize: 12,
                                      ),
                                    ),
                                    if (!current && !resolved)
                                      const Padding(
                                        padding: EdgeInsets.only(top: 8),
                                        child: Text(
                                          'Open and refresh this diff. If it changed, select a new line to create a current finding; this original note stays readable.',
                                        ),
                                      ),
                                    if ((f['submissions'] as List).isNotEmpty)
                                      SelectableText(
                                        'Input receipts: ${(f['submissions'] as List).join(', ')}',
                                        style: const TextStyle(fontSize: 12),
                                      ),
                                    if (resolved)
                                      SelectableText(
                                        'Verified: ${f['resolution']['note']}',
                                      ),
                                    Wrap(
                                      spacing: 8,
                                      children: [
                                        TextButton(
                                          onPressed: locked
                                              ? null
                                              : () => git.openDiff(
                                                  w,
                                                  basis['path'] as String,
                                                  basis['basis'] as String,
                                                  commit:
                                                      basis['commit']
                                                          as String?,
                                                ),
                                          child: const Text('Open diff'),
                                        ),
                                        if (!resolved)
                                          TextButton(
                                            onPressed: locked || !eligible
                                                ? null
                                                : () => resolve(f, diff),
                                            child: const Text(
                                              'Resolve after inspection',
                                            ),
                                          ),
                                        if (resolved)
                                          TextButton(
                                            onPressed: locked
                                                ? null
                                                : () => git.editFinding(w, {
                                                    'action': 'reopen',
                                                    'id': f['id'],
                                                    'revision': f['revision'],
                                                  }),
                                            child: const Text('Reopen'),
                                          ),
                                        TextButton(
                                          onPressed: locked
                                              ? null
                                              : () => git.editFinding(w, {
                                                  'action': 'remove',
                                                  'id': f['id'],
                                                  'revision': f['revision'],
                                                }),
                                          child: const Text('Remove note'),
                                        ),
                                      ],
                                    ),
                                  ],
                                ),
                              ),
                            );
                          },
                        ),
                      if (state.cursor != null)
                        TextButton(
                          onPressed: locked
                              ? null
                              : () => git.loadFindings(w, more: true),
                          child: const Text('Load more findings'),
                        ),
                    ],
                  ),
                ),
                Wrap(
                  spacing: 8,
                  runSpacing: 8,
                  children: [
                    TextButton(
                      onPressed: locked ? null : () => git.loadFindings(w),
                      child: const Text('Refresh findings'),
                    ),
                    FilledButton(
                      onPressed: locked || state.selected.isEmpty
                          ? null
                          : () => submit(false),
                      child: const Text('Queue selected'),
                    ),
                    OutlinedButton(
                      onPressed: locked || state.selected.isEmpty
                          ? null
                          : () => submit(true),
                      child: const Text('Steer with selected'),
                    ),
                  ],
                ),
              ],
            ),
          ),
        ),
      );
    },
  );
}
