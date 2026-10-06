import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/file_host.dart';

import 'app_host_test.dart' show HostBridge;

class FileBridge extends HostBridge {
  final docs = <String, Map<String, dynamic>>{};
  final editorCalls = <Map<String, dynamic>>[];
  Completer<void>? opening;
  @override
  Future<dynamic> call(Map<String, dynamic> cmd) async {
    if (cmd['command'] != 'editor') return super.call(cmd);
    final r = Map<String, dynamic>.from(cmd['request'] as Map);
    editorCalls.add(r);
    final session = cmd['session'];
    if (r['action'] == 'workspace') {
      if (opening != null) await opening!.future;
      if (session == 'missing') {
        throw StateError('Working folder is unavailable.');
      }
      return {
        'project': session,
        'root': 'C:/$session',
        'layout': null,
        'recovery': [],
        'documents': [],
      };
    }
    if (r['action'] == 'tree') {
      return {
        'entries': [
          {'name': 'nested', 'path': 'nested', 'directory': true},
          {'name': 'a.txt', 'path': 'a.txt', 'directory': false},
        ],
        'cursor': r['cursor'] == 0 ? 200 : null,
      };
    }
    if (r['action'] == 'open') {
      return docs.putIfAbsent(
        '${r['project']}/${r['path']}',
        () => {
          'document': '${r['project']}/${r['path']}',
          'project': r['project'],
          'snapshot': {
            'path': r['path'],
            'text': 'saved',
            'readonly': false,
            'revision': 'digest',
            'newline': 'lf',
            'bom': false,
          },
          'text': 'saved',
          'version': 0,
          'dirty': false,
        },
      );
    }
    if (r['action'] == 'close') {
      docs.remove(r['document']);
      return null;
    }
    return null;
  }
}

void main() {
  test(
    'lazy paging and A B A retain document identity; side chats clear context',
    () async {
      final bridge = FileBridge();
      final files = FileHost(bridge);
      await files.bind('A', 'C:/A');
      final a = files.selected!;
      expect(bridge.editorCalls.where((r) => r['action'] == 'tree').length, 1);
      expect(a.tree['nested'], isNull);
      files.expand(a, 'nested');
      await Future<void>.delayed(Duration.zero);
      expect(a.tree.containsKey('nested'), true);
      final doc = await files.open(a, 'a.txt');
      doc!.text = 'retained draft';
      await files.load(a, '.');
      expect(a.tree['.']!.entries.length, 4);
      expect(a.tree['.']!.cursor, isNull);
      await files.bind('B', 'C:/B');
      await files.open(files.selected!, 'b.txt');
      await files.bind('A', 'C:/A');
      expect(identical(files.active, doc), true);
      expect(files.active!.text, 'retained draft');
      await files.bind('side', null);
      expect(files.selected, isNull);
      expect(files.documents.length, 2);
      files.dispose();
    },
  );
  test('repeated in-flight binding coalesces; unavailable folder retains documents for recovery', () async {
    final bridge = FileBridge()..opening = Completer<void>();
    final files = FileHost(bridge);
    final pending = files.bind('A', 'C:/A');
    await files.bind('A', 'C:/A');
    expect(
      bridge.editorCalls.where((r) => r['action'] == 'workspace').length,
      1,
    );
    bridge.opening!.complete();
    await pending;
    bridge.opening = null;
    final a = files.selected!;
    await files.open(a, 'a.txt');
    await files.bind('missing', 'C:/missing');
    expect(files.selected, isNull);
    expect(files.error, contains('unavailable'));
    expect(files.documents.length, 1);
    await files.bind('A', 'C:/A');
    expect(files.active!.path, 'a.txt');
    files.dispose();
  });
}
