import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:flutter_test/flutter_test.dart';

class HistoryMutationBridge implements ChatBridge {
  final requests = <Map<String, dynamic>>[];
  final jobs = <String, Map<String, dynamic>>{};
  String head = 'old';
  bool failHistory = false, failApply = false;
  Map<String, dynamic>? operation;

  Map<String, dynamic> get status => {
    'repo': 'A',
    'head': head,
    'revision': head,
    'branch': 'main',
    'entries': [],
  };

  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = Map<String, dynamic>.from(command['request'] as Map);
    if (request['action'] != 'poll') {
      requests.add(request);
      final id = '${requests.length}';
      jobs[id] = request;
      return {'job': id};
    }
    final job = jobs[request['job']]!;
    final dynamic value;
    switch (job['action']) {
      case 'status':
        value = status;
      case 'history':
        if (failHistory) {
          throw StateError('History unavailable. Refresh history.');
        }
        value = {
          'items': [
            if (job['head'] == 'new') {'id': 'new', 'subject': 'New commit'},
            {'id': 'old', 'subject': 'Previous commit'},
          ],
          'next': null,
        };
      case 'review':
        operation = Map<String, dynamic>.from(job['operation'] as Map);
        value = {
          'repo': 'A',
          'token': 'once',
          'paths': [],
          'operation': operation,
        };
      case 'apply':
        if (failApply) throw StateError('Commit hook refused. Draft remains.');
        if (operation?['kind'] != 'stage') {
          head = 'new';
        }
        value = {'completed': true, 'operation': operation, 'status': status};
      case 'discardReview':
        value = {};
      default:
        throw StateError('Unexpected fixture action: ${job['action']}');
    }
    return {'done': true, 'value': value};
  }
}

void main() {
  test(
    'Staging retains the loaded history page when HEAD is unchanged',
    () async {
      final bridge = HistoryMutationBridge();
      final host = GitHost(bridge);
      addTearDown(host.dispose);
      await host.bind('A', 'C:/A');
      final w = host.selected!;
      w.history.add({'id': 'older', 'subject': 'Loaded older page'});
      w.historyNext = 60;
      final previousHistory = w.history;
      await host.changeIndex(w, {
        'kind': 'stage',
        'paths': ['notes.txt'],
      });
      expect(w.history, same(previousHistory));
      expect(w.history.map((row) => row['id']), ['old', 'older']);
      expect(w.historyNext, 60);
      expect(w.historyHead, 'old');
      expect(
        bridge.requests.where((r) => r['action'] == 'history'),
        hasLength(1),
      );
    },
  );

  test(
    'Successful commit shows the new history row without manual refresh',
    () async {
      final bridge = HistoryMutationBridge();
      final host = GitHost(bridge);
      addTearDown(host.dispose);
      await host.bind('A', 'C:/A');
      final w = host.selected!..commitDraft = 'New commit';
      w.expandedCommits.add('old');
      w.commitFileCache['old'] = {
        'files': ['notes.txt'],
      };
      final oldTab = {
        'commit': 'old',
        'path': 'notes.txt',
        'patch': 'old contents',
      };
      w.tabs['old:notes.txt'] = oldTab;
      w.activeTab = 'old:notes.txt';
      final preview = (await host.review(w, {
        'kind': 'commitAll',
        'message': w.commitDraft,
      }))!;
      await host.resolveReview(w, preview, apply: true);
      expect(w.history.map((row) => row['id']), ['new', 'old']);
      expect(w.historyHead, 'new');
      expect(w.commitDraft, isEmpty);
      expect(w.notice, 'Git action completed.');
      expect(w.expandedCommits, contains('old'));
      expect(w.commitFileCache['old']!['files'], ['notes.txt']);
      expect(w.tabs['old:notes.txt'], same(oldTab));
      expect(w.activeTab, 'old:notes.txt');
      expect(w.reviewOpen, isNull);
      expect(w.busy, isFalse);
    },
  );

  test('Unavailable history after commit preserves success and permits read-only retry', () async {
    final bridge = HistoryMutationBridge();
    final host = GitHost(bridge);
    addTearDown(host.dispose);
    await host.bind('A', 'C:/A');
    final w = host.selected!..commitDraft = 'New commit';
    final previousHistory = w.history;
    bridge.failHistory = true;
    final preview = (await host.review(w, {
      'kind': 'commit',
      'message': w.commitDraft,
    }))!;
    await host.resolveReview(w, preview, apply: true);
    expect(w.status!['head'], 'new');
    expect(w.history, same(previousHistory));
    expect(w.error, contains('History unavailable'));
    expect(w.notice, 'Git action completed.');
    expect(w.commitDraft, isEmpty);
    expect(w.reviewOpen, isNull);
    expect(w.busy, isFalse);
    bridge.failHistory = false;
    await host.loadHistory(w);
    expect(w.history.map((row) => row['id']), ['new', 'old']);
    expect(w.error, isNull);
    expect(bridge.requests.where((r) => r['action'] == 'apply'), hasLength(1));
  });

  test('Cancel and hook refusal retain history and commit draft without another history read', () async {
    final bridge = HistoryMutationBridge();
    final host = GitHost(bridge);
    addTearDown(host.dispose);
    await host.bind('A', 'C:/A');
    final w = host.selected!..commitDraft = 'Keep this message';
    final previousHistory = w.history;
    final cancel = (await host.review(w, {'kind': 'commit'}))!;
    await host.resolveReview(w, cancel, apply: false);
    bridge.failApply = true;
    final refused = (await host.review(w, {'kind': 'commit'}))!;
    await host.resolveReview(w, refused, apply: true);
    expect(w.history, same(previousHistory));
    expect(w.historyHead, 'old');
    expect(w.commitDraft, 'Keep this message');
    expect(w.error, contains('hook refused'));
    expect(
      bridge.requests.where((r) => r['action'] == 'history'),
      hasLength(1),
    );
    expect(w.reviewOpen, isNull);
    expect(w.busy, isFalse);
  });
}
