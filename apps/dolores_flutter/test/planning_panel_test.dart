import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/composer_commands.dart';
import 'package:dolores_flutter/planning_panel.dart';

import 'instruction_queue_test.dart' show QueueBridge;
import 'support/render_fonts.dart';

class PlanBridge extends QueueBridge {
  Map<String, dynamic> plan = {
    'revision': 1,
    'planning': true,
    'draft': 'Inspect the current parser, make a bounded change and run the focused checks.',
    'decisions': 'Preserve existing files.',
    'references': [],
  };
  bool failPlan = false, stale = false;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'composerCommands') {
      return {'commands': [], 'skills': [], 'selected': []};
    }
    if (c['command'] == 'planning') return Map.of(plan);
    if (c['command'] == 'editPlanning') {
      if (failPlan) throw StateError('Storage unavailable. Your edits remain.');
      final e = c['edit'] as Map;
      if (e['action'] == 'draft') {
        plan = {
          ...plan,
          'draft': e['text'],
          'decisions': e['decisions'],
          'accepted': null,
        };
      }
      if (e['action'] == 'accept') {
        plan = {
          ...plan,
          'planning': true,
          'accepted': {'revision': plan['revision'], 'goal': null},
        };
      }
      if (e['action'] == 'add') {
        if (stale) {
          throw StateError(
            'Source changed. Preview again; your draft remains.',
          );
        }
        plan = {
          ...plan,
          'references': [
            {
              'kind': 'file',
              'source': 'public.txt',
              'revision': 'f' * 64,
              'text': 'PUBLIC_SNAPSHOT',
              'omissions': '',
            },
          ],
        };
      }
      plan = {...plan, 'revision': (plan['revision'] as int) + 1};
      return plan;
    }
    if (c['command'] == 'referenceChoices') {
      return {
        'choices': [
          {
            'title': 'public.txt',
            'source': {'kind': 'file', 'path': 'public.txt'},
          },
        ],
      };
    }
    if (c['command'] == 'referencePreview') {
      return {
        'kind': 'file',
        'source': 'public.txt',
        'revision': 'f' * 64,
        'text': 'PUBLIC_SNAPSHOT',
        'omissions': '',
      };
    }
    if (c['command'] == 'startPlan') {
      starts++;
      return null;
    }
    return super.call(c);
  }
}

ChatController controller(PlanBridge b) => ChatController(b)
  ..session = 'audit'
  ..loading = false
  ..configured = true
  ..planningAvailable = true;
void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(loadReviewFonts);
  test('at picker respects plain boundaries, selection and IME', () {
    TextEditingValue v(String s) => TextEditingValue(
      text: s,
      selection: TextSelection.collapsed(offset: s.length),
    );
    expect(commandQuery(v('@file')), isNotNull);
    expect(commandQuery(v('person@example')), isNull);
    expect(commandQuery(v('```\n@file')), isNull);
    expect(
      commandQuery(
        v('@file').copyWith(composing: const TextRange(start: 0, end: 2)),
      ),
      isNull,
    );
  });
  testWidgets(
    'failed plan save retains edits; accept and reopen never start execution',
    (tester) async {
      final b = PlanBridge()..failPlan = true;
      final chat = controller(b)..draft = 'User draft';
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      final dialog = showPlan(tester.element(find.byType(ChatPage)), chat);
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('plan-text')),
        'Inspect then verify the public report.',
      );
      await tester.tap(find.byKey(const Key('save-plan')));
      await tester.pumpAndSettle();
      expect(
        find.text('Inspect then verify the public report.'),
        findsOneWidget,
      );
      expect(chat.draft, 'User draft');
      expect(b.starts, 0);
      b.failPlan = false;
      await tester.tap(find.byKey(const Key('save-plan')));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('accept-plan')));
      await tester.pumpAndSettle();
      expect(b.starts, 0);
      await chat.refreshPlanning();
      expect(b.starts, 0);
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await dialog;
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'at picker keyboard selection previews then shares without sending',
    (tester) async {
      final b = PlanBridge();
      final chat = controller(b)..commandPickerAvailable = true;
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        '@file',
      );
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('command-reference-file')), findsOneWidget);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      await tester.tap(find.text('public.txt'));
      await tester.pumpAndSettle();
      expect(find.text('PUBLIC_SNAPSHOT'), findsOneWidget);
      await tester.tap(find.byKey(const Key('share-context')));
      await tester.pumpAndSettle();
      expect(chat.planning['references'], isNotEmpty);
      expect(b.starts, 0);
      expect(chat.draft, isEmpty);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'stale source selection retains preview and draft; reselection succeeds without sending',
    (tester) async {
      final b = PlanBridge()..stale = true;
      final chat = controller(b)..draft = 'Keep my unsent text';
      await tester.pumpWidget(DoloresApp(chat: chat));
      await tester.pumpAndSettle();
      final dialog = showContextPicker(
        tester.element(find.byType(ChatPage)),
        chat,
        'file',
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('public.txt'));
      await tester.pumpAndSettle();
      expect(find.text('PUBLIC_SNAPSHOT'), findsOneWidget);
      await tester.tap(find.byKey(const Key('share-context')));
      await tester.pumpAndSettle();
      expect(chat.draft, 'Keep my unsent text');
      expect(find.textContaining('Source changed'), findsWidgets);
      b.stale = false;
      await tester.tap(find.byKey(const Key('share-context')));
      await tester.pumpAndSettle();
      expect(await dialog, true);
      expect(b.starts, 0);
      expect(chat.planning['references'], isNotEmpty);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  for (final dark in [false, true]) {
    testWidgets('plan review compact ${dark ? 'dark' : 'light'}', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(540, 900);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = controller(PlanBridge());
      final key = GlobalKey();
      await tester.pumpWidget(
        RepaintBoundary(
          key: key,
          child: DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        ),
      );
      await tester.pumpAndSettle();
      final dialog = showPlan(tester.element(find.byType(ChatPage)), chat);
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
      final boundary =
          key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
      await tester.runAsync(() async {
        final image = await boundary.toImage(pixelRatio: 1);
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        image.dispose();
        File('../../output/harness/eh04-${dark ? 'dark' : 'light'}-compact.png')
            .writeAsBytesSync(bytes!.buffer.asUint8List());
      });
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await dialog;
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    });
  }
}
