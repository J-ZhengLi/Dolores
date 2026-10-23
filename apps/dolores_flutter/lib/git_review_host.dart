part of 'git_host.dart';

class GitReviewState {
  final items = <String, Map<String, dynamic>>{};
  final selected = <String>{};
  int? cursor;
  bool loading = false, loaded = false;
  String? error, notice, job;
  Map<String, dynamic>? pending;
}

extension GitReviewOperations on GitHost {
  void _acceptSubmission(GitReviewState state, Map submission) {
    for (final selection in submission['selected'] as List) {
      final old = state.items[selection['id']];
      if (old != null && old['revision'] == selection['revision']) {
        state.items[selection['id'] as String] = {
          ...old,
          'revision': (selection['revision'] as int) + 1,
          'submissions': [...old['submissions'] as List, submission['id']],
        };
      }
    }
    state.selected.clear();
  }

  bool _sameFields(Map left, Map right) =>
      left.length == right.length &&
      left.keys.every((key) => left[key] == right[key]);

  bool _savedMatches(Map saved, Map operation) {
    if (operation['action'] == 'submit') {
      final expected = operation['submission'] as Map;
      final actual = saved['selected'] as List?;
      final selected = expected['selected'] as List;
      return saved['id'] == expected['id'] &&
          saved['mode'] == expected['mode'] &&
          actual != null &&
          actual.length == selected.length &&
          List.generate(
            selected.length,
            (i) => _sameFields(actual[i] as Map, selected[i] as Map),
          ).every((v) => v);
    }
    if (saved['id'] != operation['id']) return false;
    if (operation['action'] == 'capture') {
      final anchor = saved['anchor'] as Map?;
      return saved['text'] == operation['text'] &&
          anchor != null &&
          anchor['side'] == operation['side'] &&
          anchor['line'] == operation['line'] &&
          _sameFields(
            anchor['comparison'] as Map,
            operation['comparison'] as Map,
          );
    }
    final evidence = saved['resolution'] as Map?;
    return saved['revision'] == (operation['revision'] as int) + 1 &&
        evidence != null &&
        evidence['note'] == operation['note'] &&
        _sameFields(
          evidence['comparison'] as Map,
          operation['comparison'] as Map,
        );
  }

  GitReviewState reviewState(GitWorkspace w, [String? session]) =>
      w.reviewStates.putIfAbsent(session ?? w.session, GitReviewState.new);

  Future<dynamic> reviewLocal(String session, Map<String, dynamic> request) =>
      bridge.call({
        'command': 'reviewFindings',
        'session': session,
        'request': request,
      });

  Future<void> loadFindings(GitWorkspace w, {bool more = false}) async {
    final session = w.session;
    final state = reviewState(w, session);
    if (state.loading) return;
    state.loading = true;
    state.error = null;
    changed();
    try {
      final page = await reviewLocal(session, {
        'action': 'list',
        if (more) 'cursor': state.cursor,
      });
      if (!more) state.items.clear();
      for (final item in page['items'] as List) {
        state.items[item['id'] as String] = Map<String, dynamic>.from(item);
      }
      state.cursor = page['nextCursor'] as int?;
      state.loaded = true;
      state.selected.removeWhere((id) => !state.items.containsKey(id));
    } catch (e) {
      state.error = '$e';
    } finally {
      state.loading = false;
      changed();
    }
  }

  Map<String, dynamic> comparisonFor(GitWorkspace w, Map diff) => {
    'root': w.status!['root'],
    'repository': w.status!['repo'],
    'path': diff['path'],
    'basis': diff['basis'],
    'commit': diff['commit'],
    'revision': diff['revision'],
    'digest': diff['digest'],
  };

  bool findingCurrent(GitWorkspace w, Map finding, Map? diff) {
    if (diff == null || w.status == null) return false;
    final anchor = finding['anchor']['comparison'] as Map;
    final current = comparisonFor(w, diff);
    return current.keys.every((k) => current[k] == anchor[k]) &&
        diff['revision'] == w.status!['revision'];
  }

  Future<bool> editFinding(GitWorkspace w, Map<String, dynamic> request) async {
    final session = w.session;
    final state = reviewState(w, session);
    if (state.loading || state.pending != null) return false;
    state.loading = true;
    state.error = null;
    changed();
    try {
      await reviewLocal(session, request);
      return true;
    } catch (e) {
      state.error = '$e';
      return false;
    } finally {
      state.loading = false;
      // Preserve the error while refreshing stale revisions.
      final error = state.error;
      if (w.session == session) await loadFindings(w);
      state.error = error ?? state.error;
      changed();
    }
  }

  Future<bool> runFinding(
    GitWorkspace w,
    Map<String, dynamic> operation,
  ) async {
    final state = reviewState(w);
    if (w.busy || state.loading || state.pending != null) return false;
    state.pending = Map.of(operation);
    state.error = null;
    state.notice = null;
    w.busy = true;
    w.jobSession = w.session;
    changed();
    return checkFinding(w);
  }

  Future<bool> checkFinding(GitWorkspace w) async {
    final session = w.jobSession ?? w.session;
    final state = reviewState(w, session);
    final operation = state.pending;
    if (operation == null || state.loading) return false;
    state.loading = true;
    state.error = null;
    changed();
    bool success = false;
    try {
      if (state.job == null) {
        final start = await bridge.call({
          'command': 'git',
          'session': session,
          'request': {'action': 'finding', 'operation': operation},
        });
        state.job = start['job'] as String;
        w.job = state.job;
      }
      while (!_disposed) {
        final poll = await bridge.call({
          'command': 'git',
          'session': session,
          'request': {'action': 'poll', 'job': state.job},
        });
        if (poll['done'] != true) {
          await Future<void>.delayed(const Duration(milliseconds: 40));
          continue;
        }
        if (poll['error'] != null) {
          state.error = '${poll['error']}';
        } else {
          final value = poll['value'] as Map;
          if (value['finding'] is Map) {
            final finding = Map<String, dynamic>.from(value['finding']);
            state.items[finding['id'] as String] = finding;
          }
          state.notice = operation['action'] == 'submit'
              ? 'Feedback accepted. Inspect delivery in the chat; resolution stays separate.'
              : operation['action'] == 'capture'
              ? 'Finding saved.'
              : 'Resolution saved with the inspected comparison.';
          if (operation['action'] == 'submit') {
            _acceptSubmission(state, operation['submission'] as Map);
          }
          success = true;
        }
        state.pending = null;
        state.job = null;
        w.job = null;
        w.jobSession = null;
        w.busy = false;
        break;
      }
    } catch (e) {
      state.error =
          '$e Status is uncertain. Check status before starting another review operation; saved notes and draft remain.';
      // A lost poll may follow a committed result. Reconcile via read-only records.
      try {
        final submit = operation['action'] == 'submit';
        final id = submit ? operation['submission']['id'] : operation['id'];
        final saved = await reviewLocal(session, {
          'action': submit ? 'submission' : 'get',
          'id': id,
        });
        final confirmed = saved is Map && _savedMatches(saved, operation);
        if (confirmed) {
          if (state.job != null) {
            final released = await bridge.call({
              'command': 'git',
              'session': session,
              'request': {'action': 'releaseFindingJob', 'job': state.job},
            });
            if (released['released'] != true) {
              state.notice = 'Saved result found. Check status again to finish releasing its job; the operation will not be repeated.';
              return false;
            }
          }
          if (!submit) {
            state.items[saved['id'] as String] = Map<String, dynamic>.from(
              saved,
            );
          }
          state.error = null;
          state.notice =
              'Saved result inspected. The operation was not repeated.';
          state.pending = null;
          state.job = null;
          w.job = null;
          w.jobSession = null;
          w.busy = false;
          if (submit) _acceptSubmission(state, operation['submission'] as Map);
          success = true;
        }
      } catch (_) {
        /* Keep the original handle and explicit recovery error. */
      }
    } finally {
      state.loading = false;
      changed();
    }
    if (success && operation['action'] == 'submit') {
      try {
        await afterReviewSubmission?.call(session);
      } catch (e) {
        state.error =
            'Feedback is saved. Open its chat to inspect delivery: $e';
        changed();
      }
    }
    return success;
  }
}
