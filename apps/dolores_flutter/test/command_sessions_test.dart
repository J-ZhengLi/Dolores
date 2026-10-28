import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/command_sessions.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/render_fonts.dart';
import 'frost_design_test.dart' show capture, viewport;

class CommandsBridge implements ChatBridge {
  final calls = <Map<String, dynamic>>[];
  bool failInput = false, failRead = false, unsaved = false, attached = true;
  Completer<void>? waitInput;
  bool cleanupPending = false, failCleanup = false;
  final receipt = <String, dynamic>{
    'id': 'command-one',
    'task': 'chat-one',
    'run': 'run-one',
    'program': 'node',
    'execution': 'native; not sandboxed',
    'state': 'running',
    'exitCode': null,
    'timeoutSeconds': 30,
    'captureBytes': 8192,
    'outputBytes': 17,
    'inputRevision': 0,
    'inputBytes': 0,
    'stdinClosed': false,
    'stdinClosing': false,
    'cleanupComplete': false,
    'lastInput': null,
  };
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command);
    expect(command['command'], 'commandSession');
    expect(command['session'], 'chat-one');
    final request = command['request'] as Map;
    switch (request['operation']) {
      case 'list':
        return {
          'isolation': {
            'available': true,
            'pendingCleanup': cleanupPending ? 1 : 0,
          },
          'items': [Map.of(receipt)],
          'nextCursor': null,
          'liveItems': attached ? [Map.of(receipt)] : [],
          'retainedRuns': unsaved
              ? [
                  {'run': 'run-one', 'unsaved': true},
                ]
              : [],
          'storageWarnings': unsaved
              ? [
                  {'run': 'run-one', 'message': 'Storage unavailable.'},
                ]
              : [],
        };
      case 'read':
        if (failRead) throw 'Output read unavailable. Refresh to retry.';
        final offset = request['offset'] as int;
        return {
          'receipt': Map.of(receipt),
          'offset': offset,
          'nextOffset': offset == 0 ? 6 : 17,
          'hasMore': offset == 0,
          'output': offset == 0 ? 'Ready\n' : 'Result 123\n',
          'lossyUtf8': false,
        };
      case 'input':
        await waitInput?.future;
        if (failInput) throw 'Input revision changed. Nothing was sent.';
        expect(request['id'], 'command-one');
        expect(request['run'], 'run-one');
        expect(request['revision'], receipt['inputRevision']);
        receipt['inputRevision'] = (receipt['inputRevision'] as int) + 1;
        if (request['eof'] == true) receipt['stdinClosed'] = true;
        receipt['lastInput'] = {
          'state': 'written',
          'bytes': 3,
          'writtenBytes': 3,
        };
        return {'session': Map.of(receipt)};
      case 'interrupt':
        receipt['state'] = 'interrupted';
        receipt['cleanupComplete'] = true;
        return {'session': Map.of(receipt)};
      case 'close':
        attached = false;
        return {'session': Map.of(receipt)};
      case 'save':
        unsaved = false;
        return {'saved': true};
      case 'recoverIsolation':
        if (failCleanup) {
          throw 'Cleanup path changed. Restore the reviewed folder and retry.';
        }
        cleanupPending = false;
        return {'available': true, 'pendingCleanup': 0};
      default:
        throw 'Unexpected operation';
    }
  }
}

Future<ChatController> open(
  WidgetTester tester,
  CommandsBridge bridge, {
  bool dark = true,
  double width = 420,
  GlobalKey? renderKey,
}) async {
  tester.view.physicalSize = Size(width, 620);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  final chat = ChatController(bridge)
    ..session = 'chat-one'
    ..draft = 'Keep the conversation draft';
  addTearDown(chat.dispose);
  await tester.pumpWidget(
    RepaintBoundary(
      key: renderKey,
      child: MaterialApp(
        debugShowCheckedModeBanner: false,
        theme: doloresTheme(dark),
        home: Scaffold(
          body: Builder(
            builder: (context) => TextButton(
              onPressed: () => showCommandSessions(context, chat),
              child: const Text('Open commands'),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open commands'));
  await tester.pumpAndSettle();
  return chat;
}

void main() {
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets('Named command environment review dark=$dark width=$width', (
        tester,
      ) async {
        viewport(tester, Size(width, 800));
        await tester.runAsync(loadReviewFonts);
        final chat = ChatController(CommandsBridge())
          ..workspaceRoot = r'C:\public\work'
          ..toolApproval = {
            'name': 'run_command',
            'callId': 'ssh-env',
            'target': 'python',
            'command': {
              'executable': r'C:\runtime\python.exe',
              'invocation': {
                'args': [
                  'ssh_connect.py',
                  '--password-env',
                  'NAS_ROOT_PASSWORD',
                ],
              },
              'inherit_env': ['NAS_ROOT_PASSWORD'],
            },
          };
        addTearDown(chat.dispose);
        final key = GlobalKey();
        await tester.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: RepaintBoundary(
                key: key,
                child: ColoredBox(
                  color: Palette(dark).bg,
                  child: ToolApprovalCard(chat: chat),
                ),
              ),
            ),
          ),
        );
        await tester.pumpAndSettle();
        expect(
          find.text('Environment variables to share with this command'),
          findsOneWidget,
        );
        expect(find.text('NAS_ROOT_PASSWORD'), findsOneWidget);
        expect(
          tester.getRect(find.byKey(const Key('allow-tool'))).bottom,
          lessThanOrEqualTo(800),
        );
        expect(tester.takeException(), isNull);
        await capture(tester, key, 'ssh-environment-$dark-${width.toInt()}');
        await tester.pumpWidget(const SizedBox());
      });
    }
  }
  testWidgets(
    'Cleanup failure preserves drafts and output, and only explicit retry recovers',
    (tester) async {
      final bridge = CommandsBridge()
        ..cleanupPending = true
        ..failCleanup = true;
      final chat = await open(tester, bridge);
      await tester.enterText(
        find.byKey(const Key('command-input')),
        'Keep stdin draft',
      );
      await tester.pump(const Duration(seconds: 2));
      expect(
        bridge.calls.where(
          (c) => c['request']['operation'] == 'recoverIsolation',
        ),
        isEmpty,
      );
      await tester.ensureVisible(
        find.byKey(const Key('retry-isolation-cleanup')),
      );
      await tester.tap(find.byKey(const Key('retry-isolation-cleanup')));
      await tester.pumpAndSettle();
      expect(find.textContaining('Cleanup path changed'), findsOneWidget);
      expect(find.text('Ready\n'), findsOneWidget);
      expect(chat.commandInputDrafts.values, contains('Keep stdin draft'));
      bridge.failCleanup = false;
      await tester.ensureVisible(
        find.byKey(const Key('retry-isolation-cleanup')),
      );
      await tester.tap(find.byKey(const Key('retry-isolation-cleanup')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('retry-isolation-cleanup')), findsNothing);
      expect(
        bridge.calls
            .where((c) => c['request']['operation'] == 'recoverIsolation')
            .length,
        2,
      );
      expect(chat.draft, 'Keep the conversation draft');
      await tester.pumpWidget(const SizedBox());
    },
  );
  testWidgets(
    'Isolated start review shows exact grants and keeps Run once reachable',
    (tester) async {
      tester.view.physicalSize = const Size(360, 500);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(CommandsBridge())
        ..workspaceRoot = r'C:\public\work'
        ..toolApproval = {
          'name': 'command_session',
          'callId': 'start-isolated',
          'target': 'node',
          'query': '{"operation":"start","isolation":"isolated"}',
          'command': {
            'executable': r'C:\runtime\node.exe',
            'invocation': {
              'args': ['build.cjs'],
            },
            'isolation': {
              'workspace': r'C:\public\work',
              'readonly_roots': [r'C:\public\reference'],
            },
          },
        };
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: Scaffold(body: ToolApprovalCard(chat: chat)),
        ),
      );
      expect(find.textContaining('network denied'), findsOneWidget);
      expect(find.textContaining(r'C:\public\reference'), findsOneWidget);
      expect(
        find.textContaining('Runs with your OS permissions'),
        findsNothing,
      );
      expect(
        tester.getRect(find.byKey(const Key('allow-tool'))).bottom,
        lessThan(500),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
  setUpAll(() async {
    if (const String.fromEnvironment('AUDIT_RENDER_DIR').isNotEmpty) {
      await loadReviewFonts();
    }
  });
  testWidgets(
    'Model stdin review names the exact effect and keeps decision buttons reachable',
    (tester) async {
      tester.view.physicalSize = const Size(360, 500);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(CommandsBridge())
        ..toolApproval = {
          'name': 'command_session',
          'callId': 'input-one',
          'target': 'command-one',
          'query': '{"operation":"input","session":"command-one","revision":2,"text":"deploy\\n","eof":false}',
        };
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: Scaffold(body: ToolApprovalCard(chat: chat)),
        ),
      );
      expect(find.text('Send this input to the command?'), findsOneWidget);
      expect(
        find.text('Exact command operation and input (JSON):'),
        findsOneWidget,
      );
      expect(find.textContaining('deploy\\n'), findsOneWidget);
      expect(find.textContaining('delivery to stdin'), findsOneWidget);
      expect(
        tester.getRect(find.byKey(const Key('allow-tool'))).bottom,
        lessThan(500),
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
  for (final dark in [false, true]) {
    testWidgets(
      'Exact input, interruption and saved output recover in compact UI ($dark)',
      (tester) async {
        final bridge = CommandsBridge();
        final renderKey = GlobalKey();
        final chat = await open(
          tester,
          bridge,
          dark: dark,
          renderKey: renderKey,
        );
        expect(find.text('Ready\n'), findsOneWidget);
        await tester.enterText(find.byKey(const Key('command-input')), '你好\n');
        await tester.tap(find.text('Close').first);
        await tester.pumpAndSettle();
        await tester.tap(find.text('Open commands'));
        await tester.pumpAndSettle();
        expect(
          tester
              .widget<TextField>(find.byKey(const Key('command-input')))
              .controller!
              .text,
          '你好\n',
        );
        await tester.tap(find.byKey(const Key('command-send')));
        await tester.pumpAndSettle();
        final sent = bridge.calls
            .where((c) => c['request']['operation'] == 'input')
            .single;
        expect(sent['request']['text'], '你好\n');
        expect(
          tester
              .widget<TextField>(find.byKey(const Key('command-input')))
              .controller!
              .text,
          '',
        );
        await tester.ensureVisible(find.text('More output'));
        await tester.tap(find.text('More output'));
        await tester.pumpAndSettle();
        expect(find.text('Ready\nResult 123\n'), findsOneWidget);
        await tester.tap(find.text('Refresh'));
        await tester.pumpAndSettle();
        expect(find.text('Ready\nResult 123\n'), findsOneWidget);
        await tester.tap(find.text('Close stdin'));
        await tester.pumpAndSettle();
        final eof = bridge.calls
            .where((c) => c['request']['operation'] == 'input')
            .last['request'];
        expect(eof['text'], '');
        expect(eof['eof'], true);
        expect(
          tester
              .widget<TextField>(find.byKey(const Key('command-input')))
              .readOnly,
          true,
        );
        await tester.tap(find.text('Interrupt'));
        await tester.pumpAndSettle();
        expect(
          tester
              .widget<TextField>(find.byKey(const Key('command-input')))
              .readOnly,
          true,
        );
        await tester.tap(find.text('Close command'));
        await tester.pumpAndSettle();
        expect(find.text('Ready\nResult 123\n'), findsOneWidget);
        expect(chat.draft, 'Keep the conversation draft');
        expect(tester.takeException(), isNull);
        const directory = String.fromEnvironment('AUDIT_RENDER_DIR');
        if (directory.isNotEmpty) {
          await tester.runAsync(() async {
            final boundary =
                renderKey.currentContext!.findRenderObject()
                    as RenderRepaintBoundary;
            final image = await boundary.toImage(pixelRatio: 1);
            final data = await image.toByteData(format: ui.ImageByteFormat.png);
            await Directory(directory).create(recursive: true);
            await File('$directory/commands-${dark ? 'dark' : 'light'}.png')
                .writeAsBytes(data!.buffer.asUint8List());
            image.dispose();
          });
        }
        await tester.pumpWidget(const SizedBox());
      },
    );
  }

  testWidgets(
    'Stale input and failed output reads preserve drafts and evidence; pending Send cannot repeat',
    (tester) async {
      final bridge = CommandsBridge()
        ..failInput = true
        ..unsaved = true;
      final chat = await open(tester, bridge);
      await tester.enterText(
        find.byKey(const Key('command-input')),
        'Keep this input\n',
      );
      await tester.tap(find.byKey(const Key('command-send')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('command-input')))
            .controller!
            .text,
        'Keep this input\n',
      );
      expect(find.textContaining('Input revision changed.'), findsOneWidget);
      bridge.failRead = true;
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(find.text('Ready\n'), findsOneWidget);
      expect(find.textContaining('Output read unavailable.'), findsOneWidget);
      bridge.failRead = false;
      await tester.ensureVisible(find.text('Retry saving'));
      await tester.tap(find.text('Retry saving'));
      await tester.pumpAndSettle();
      expect(bridge.unsaved, false);
      bridge.failInput = false;
      bridge.waitInput = Completer<void>();
      await tester.tap(find.byKey(const Key('command-send')));
      await tester.pump();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('command-send')))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<TextButton>(
              find.ancestor(
                of: find.text('Close').first,
                matching: find.byType(TextButton),
              ),
            )
            .onPressed,
        isNull,
      );
      await tester.pump(const Duration(seconds: 2));
      expect(
        bridge.calls.where((c) => c['request']['operation'] == 'input').length,
        2,
      );
      bridge.waitInput!.complete();
      await tester.pumpAndSettle();
      expect(chat.draft, 'Keep the conversation draft');
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
  );
}
