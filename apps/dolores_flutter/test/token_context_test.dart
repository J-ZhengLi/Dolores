import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'widget_test.dart' show ConnectionBridge;

class ReportedHistoryBridge extends ConnectionBridge {
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'messagesPage') {
      return {
        'hasOlder': false,
        'hasNewer': false,
        'items': [
          {
            'id': 2,
            'role': 'assistant',
            'content': 'reply',
            'metadata': {
              'model': 'fixture',
              'context': {
                'tokens': {'inputTokens': 99, 'contextWindowTokens': 131072},
              },
              'agent': {
                'usageByCall': [
                  {'totalTokens': 100},
                  {'totalTokens': 200},
                ],
              },
            },
          },
        ],
      };
    }
    return super.call(command);
  }
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'Per-model windows validate, preserve drafts and use a blank 128K default in compact ${dark ? 'dark' : 'light'} settings',
      (tester) async {
        tester.view.physicalSize = const Size(620, 700);
        tester.view.devicePixelRatio = 1;
        tester.platformDispatcher.platformBrightnessTestValue = dark
            ? Brightness.dark
            : Brightness.light;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        addTearDown(tester.platformDispatcher.clearPlatformBrightnessTestValue);
        final bridge = ConnectionBridge()
          ..configured = true
          ..warning = null
          ..choices = ['fixture', 'faster']
          ..contexts = {'fixture': 32768, 'faster': 8192};
        final chat = ChatController(bridge);
        await chat.initialize();
        chat.draft = 'Keep my unsent draft';
        await tester.pumpWidget(DoloresApp(chat: chat));
        await tester.tap(find.byTooltip('Conversations'));
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(const Key('connection')));
        await tester.pumpAndSettle();
        final window = find.byKey(const Key('context-window'));
        await tester.ensureVisible(window);
        expect(tester.widget<TextField>(window).controller!.text, '32768');
        await tester.enterText(window, '65536');
        final dropdown = find.byType(DropdownButtonFormField<String>);
        await tester.ensureVisible(dropdown);
        await tester.tap(dropdown);
        await tester.pumpAndSettle();
        await tester.tap(find.text('faster').last);
        await tester.pumpAndSettle();
        expect(tester.widget<TextField>(window).controller!.text, '8192');
        await tester.enterText(window, '1.5');
        await tester.tap(find.byKey(const Key('save-connection')));
        await tester.pumpAndSettle();
        expect(
          bridge.commands.where((c) => c['command'] == 'configure'),
          isEmpty,
        );
        expect(find.textContaining('must be a whole number'), findsOneWidget);
        await tester.ensureVisible(window);
        await tester.enterText(window, '');
        await tester.tap(find.byKey(const Key('save-connection')));
        await tester.pumpAndSettle();
        expect(chat.modelContexts, {'fixture': 65536, 'faster': null});
        expect(chat.draft, 'Keep my unsent draft');
        expect(
          bridge.commands.lastWhere(
            (c) => c['command'] == 'configure',
          )['modelContexts'],
          {'fixture': 65536, 'faster': null},
        );
        // The drawer remains open underneath the dialog in this compact flow.
        await tester.tap(find.byKey(const Key('connection')));
        await tester.pumpAndSettle();
        await tester.ensureVisible(window);
        expect(tester.widget<TextField>(window).controller!.text, '65536');
        await tester.enterText(window, '16384');
        await tester.tap(find.text('Cancel'));
        await tester.pumpAndSettle();
        expect(chat.modelContexts['fixture'], 65536);
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
  test(
    'Changing model and output reservation invalidates stale token previews',
    () async {
      final bridge = ConnectionBridge()
        ..configured = true
        ..choices = ['fixture', 'faster'];
      final chat = ChatController(bridge);
      addTearDown(chat.dispose);
      await chat.initialize();
      chat.draft = 'keep';
      chat.contextSummary = {
        'tokens': {'inputTokens': 123, 'contextWindowTokens': 131072},
      };
      chat.contextBasis = 'Next message preview';
      await chat.selectModel('faster');
      expect(chat.contextSummary, isNull);
      expect(chat.draft, 'keep');
      chat.contextSummary = {
        'tokens': {'inputTokens': 123},
      };
      await chat.saveRequestSettings({
        'maxOutputTokens': 128,
        'timeoutSeconds': 180,
      });
      expect(chat.contextSummary, isNull);
    },
  );
  test('Saved context uses only latest provider call total and invalidates mismatched capacity', () async {
    final bridge = ReportedHistoryBridge()..configured = true;
    final chat = ChatController(bridge);
    addTearDown(chat.dispose);
    await chat.initialize();
    await chat.select('saved');
    expect(chat.contextSummary?['reportedTokens'], 200);
    expect(chat.contextSummary?['tokens']['inputTokens'], 99);
    expect(chat.contextBasis, 'Last model call');
    chat.modelContexts = {'fixture': 32768};
    await chat.select('saved');
    expect(chat.contextSummary, isNull);
  });
}
