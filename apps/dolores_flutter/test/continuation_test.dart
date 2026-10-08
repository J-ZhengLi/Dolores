import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class PausedBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool fail = false;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    if (command['command'] == 'start' && fail) {
      throw 'Synthetic unavailable model';
    }
    if (command['command'] == 'poll') return [];
    return null;
  }
}

void main() {
  testWidgets(
    'Compact paused card preserves draft, locks duplicates and permits retry after failure in both themes',
    (tester) async {
      tester.view.physicalSize = const Size(620, 700);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final brightness in Brightness.values) {
        final bridge = PausedBridge();
        final chat = ChatController(bridge)
          ..loading = false
          ..configured = true
          ..session = 'synthetic'
          ..messages = [
            {
              'id': 1,
              'role': 'assistant',
              'content': 'Saved partial answer',
              'metadata': {
                'paused': {
                  'reason': 'outputLimit',
                  'task': 'Synthetic task',
                  'receipts': [],
                },
                'requestSettings': {
                  'maxOutputTokens': 64,
                  'timeoutSeconds': 180,
                },
                'agent': {
                  'usageByCall': [
                    {'reasoningTokens': 64, 'outputTokens': 64},
                  ],
                },
              },
            },
          ];
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: brightness == Brightness.dark
                ? ThemeMode.dark
                : ThemeMode.light,
          ),
        );
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Paused at the output limit (64 tokens)'),
          findsOneWidget,
        );
        expect(
          find.textContaining('Reasoning used 64 tokens.'),
          findsOneWidget,
        );
        await tester.enterText(
          find.byKey(const Key('composer')),
          'Keep this draft',
        );
        await tester.pump();
        expect(
          tester
              .widget<TextButton>(find.byKey(const Key('continue-task')))
              .onPressed,
          isNull,
        );
        await chat.continueTask(1);
        expect(chat.draft, 'Keep this draft');
        expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
        await tester.enterText(find.byKey(const Key('composer')), '');
        chat.configured = false;
        await chat.continueTask(1);
        expect(chat.draft, isEmpty);
        expect(chat.latestPausedId, 1);
        expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
        chat.configured = true;
        bridge.fail = true;
        await tester.pump();
        await tester.tap(find.byKey(const Key('continue-task')));
        await tester.pump();
        expect(chat.draft, isEmpty);
        expect(chat.latestPausedId, 1);
        expect(find.text('Saved partial answer'), findsOneWidget);
        bridge.fail = false;
        await tester.tap(find.byKey(const Key('continue-task')));
        await tester.pump();
        expect(chat.busy, isTrue);
        expect(
          bridge.commands.lastWhere((c) => c['command'] == 'start')['input'],
          'Continue working on the previous task.',
        );
        expect(
          bridge.commands.lastWhere(
            (c) => c['command'] == 'start',
          )['continuation'],
          1,
        );
        await chat.continueTask(1);
        expect(bridge.commands.where((c) => c['command'] == 'start').length, 2);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      }
    },
  );
}
