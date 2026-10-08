import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/file_host.dart';
import 'package:dolores_flutter/folders.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'file_host_test.dart' show FileBridge;

class PreviewBridge extends FileBridge {
  bool failOpen = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = command['request'];
    if (command['command'] == 'editor' && request['action'] == 'tree') {
      return {
        'entries': [
          for (final name in ['a.txt', 'b.txt'])
            {'name': name, 'path': name, 'directory': false},
        ],
        'cursor': null,
      };
    }
    if (command['command'] == 'editor' &&
        request['action'] == 'open' &&
        failOpen) {
      throw StateError('File is unavailable. Refresh and try again.');
    }
    return super.call(command);
  }
}

void main() {
  testWidgets('Tree double-click keeps a file when another preview opens', (
    tester,
  ) async {
    final host = AppHost(ChatController(PreviewBridge())..loading = false);
    addTearDown(host.dispose);
    await host.files.bind('A', 'C:/A');
    final workspace = host.files.selected!;
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: SizedBox(
            width: 300,
            child: FolderTree(files: host.files, openFolder: () {}),
          ),
        ),
      ),
    );
    final semantics = tester.ensureSemantics();
    expect(
      tester.getSemantics(find.text('a.txt')),
      matchesSemantics(
        label: 'a.txt',
        isButton: true,
        hasTapAction: true,
        hasFocusAction: true,
        isFocusable: true,
        hasEnabledState: true,
        isEnabled: true,
        hasSelectedState: true,
      ),
    );
    semantics.dispose();
    await tester.tap(find.text('a.txt'));
    await tester.pump(const Duration(milliseconds: 350));
    expect(workspace.layoutOwner.active.preview, 'A/a.txt');
    await tester.tap(find.text('a.txt'));
    await tester.pump(const Duration(milliseconds: 40));
    await tester.tap(find.text('a.txt'));
    await tester.pump(const Duration(milliseconds: 350));
    expect(workspace.layoutOwner.active.preview, isNull);
    await tester.tap(find.text('b.txt'));
    await tester.pump(const Duration(milliseconds: 350));
    expect(workspace.layoutOwner.active.tabs, ['A/a.txt', 'A/b.txt']);
    expect(workspace.layoutOwner.active.preview, 'A/b.txt');
    await tester.pumpWidget(const SizedBox.shrink());
    await tester.pump(const Duration(milliseconds: 300));
  });

  test(
    'Reopening a pinned file does not turn it into a replacement preview',
    () async {
      final files = FileHost(PreviewBridge());
      addTearDown(files.dispose);
      await files.bind('A', 'C:/A');
      final workspace = files.selected!;
      final first = (await files.open(workspace, 'a.txt'))!;
      workspace.layoutOwner.pin(workspace.layoutOwner.activeGroup, first.id);
      await files.open(workspace, 'b.txt');
      await files.open(workspace, 'a.txt');
      expect(workspace.layoutOwner.active.tabs, ['A/a.txt', 'A/b.txt']);
      expect(workspace.layoutOwner.active.preview, 'A/b.txt');
      expect(workspace.layoutOwner.active.active, first.id);
    },
  );

  test(
    'Failed pinned open retains tabs; retry pins only the requested file',
    () async {
      final bridge = PreviewBridge();
      final files = FileHost(bridge);
      addTearDown(files.dispose);
      await files.bind('A', 'C:/A');
      final workspace = files.selected!;
      await files.open(workspace, 'a.txt');
      bridge.failOpen = true;
      expect(await files.open(workspace, 'b.txt', preview: false), isNull);
      expect(files.error, contains('unavailable'));
      expect(workspace.layoutOwner.active.tabs, ['A/a.txt']);
      expect(workspace.layoutOwner.active.preview, 'A/a.txt');
      bridge.failOpen = false;
      await files.open(workspace, 'b.txt', preview: false);
      expect(files.error, isNull);
      expect(workspace.layoutOwner.active.tabs, ['A/b.txt']);
      expect(workspace.layoutOwner.active.preview, isNull);
      await files.open(workspace, 'a.txt');
      expect(workspace.layoutOwner.active.tabs, ['A/b.txt', 'A/a.txt']);
      expect(workspace.layoutOwner.active.preview, 'A/a.txt');
    },
  );
}
