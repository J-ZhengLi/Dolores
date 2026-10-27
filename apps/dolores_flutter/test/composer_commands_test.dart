import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/composer_commands.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/rich_composer.dart';

import 'instruction_queue_test.dart' show QueueBridge;
import 'support/render_fonts.dart';

const reference = {
  'scope': 'global',
  'name': 'precise-format',
  'revision': 2,
  'version': 1,
};

class CommandsBridge extends QueueBridge {
  String? cataloguedSession;
  int catalogueCalls = 0;
  bool failCatalogue = false;
  List<Map<String, dynamic>> selections = [];
  bool failSelection = false;
  Completer<void>? selectionGate;
  List<Map<String, dynamic>>? submittedSkills;
  String? packageId;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'start') {
      submittedSkills = [for (final selection in selections) Map.of(selection)];
    }
    if (command['command'] == 'createSession') {
      return {
        'session': {'id': 'new-project', 'title': 'New chat'},
        'workspace': {'kind': 'project', 'root': command['path']},
      };
    }
    if (command['command'] == 'composerCommands') {
      catalogueCalls++;
      cataloguedSession = command['session'] as String?;
      if (failCatalogue) {
        throw StateError('Storage unavailable');
      }
      return {
        'selected': selections,
        'commands': [
          {
            'id': 'stop',
            'title': 'Stop',
            'description': 'Stop this response',
            'available': true,
          },
          {
            'id': 'goal',
            'title': 'Goal mode',
            'description': 'Run toward a goal',
            'available': false,
            'reason': 'Not available yet',
          },
        ],
        'skills': [
          {
            'selection': {
              ...reference,
              if (packageId != null) 'packageId': packageId,
            },
            'description': 'Structure exports',
            'text': 'EXACT_PUBLIC_SNAPSHOT',
          },
          {
            'selection': {...reference, 'name': 'goal'},
            'description': 'A reviewed skill, separate from the host command',
            'text': 'GOAL_SKILL',
          },
        ],
      };
    }
    if (command['command'] == 'saveComposerSkills') {
      await selectionGate?.future;
      if (failSelection) throw StateError('Storage is locked');
      selections = List<Map<String, dynamic>>.from(
        command['selections'] as List,
      );
      return selections;
    }
    final result = await super.call(command);
    if (command['command'] == 'start' && starts > 1 && messages.isNotEmpty) {
      messages.last['metadata'] = {
        'context': {'skills': submittedSkills},
      };
    }
    return result;
  }
}

Future<ChatController> mount(
  WidgetTester tester,
  CommandsBridge bridge, {
  bool busy = false,
  bool dark = false,
  GlobalKey? boundary,
}) async {
  final chat = ChatController(bridge)
    ..loading = false
    ..configured = true
    ..instructionQueueAvailable = true
    ..commandPickerAvailable = true
    ..session = 'audit'
    ..busy = busy;
  // Picker tests need settled rows while a task is active; progress animation
  // has its own coverage in run_progress_test.dart.
  chat.reducedMotion.value = true;
  await tester.pumpWidget(
    RepaintBoundary(
      key: boundary,
      child: DoloresApp(
        chat: chat,
        themeMode: dark ? ThemeMode.dark : ThemeMode.light,
      ),
    ),
  );
  await tester.pumpAndSettle();
  addTearDown(() async {
    await tester.pumpWidget(const SizedBox());
    chat.busy = false;
    chat.dispose();
  });
  return chat;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  setUpAll(loadReviewFonts);
  test(
    'new project catalogue gets a saved workspace before reading scoped skills',
    () async {
      final bridge = CommandsBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..commandPickerAvailable = true
        ..workspaceKind = 'project'
        ..workspaceRoot = 'C:/public-fixture';
      await chat.refreshCommands();
      expect(chat.session, isNotNull);
      expect(bridge.cataloguedSession, chat.session);
      expect(chat.changing, isFalse);
      chat.dispose();
    },
  );
  testWidgets(
    'catalogue failure waits for explicit refresh while keeping the draft',
    (tester) async {
      final bridge = CommandsBridge()..failCatalogue = true;
      final chat = await mount(tester, bridge, busy: true);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        '/precise',
      );
      await tester.pumpAndSettle();
      expect(bridge.catalogueCalls, 1);
      chat.invalidateContext();
      await tester.pumpAndSettle();
      expect(bridge.catalogueCalls, 1);
      expect(chat.draft, '/precise');
      bridge.failCatalogue = false;
      await tester.tap(find.byTooltip('Refresh commands'));
      await tester.pumpAndSettle();
      expect(bridge.catalogueCalls, 2);
      expect(
        find.byKey(const Key('command-skill-precise-format')),
        findsOneWidget,
      );
    },
  );
  test('slash query respects plain boundaries, paths, fences and IME', () {
    TextEditingValue value(String text) => TextEditingValue(
      text: text,
      selection: TextSelection.collapsed(offset: text.length),
    );
    expect(commandQuery(value('/precise')), const TextRange(start: 0, end: 8));
    expect(
      commandQuery(value('Help\n/precise')),
      const TextRange(start: 5, end: 13),
    );
    expect(commandQuery(value('Use /goal')), const TextRange(start: 4, end: 9));
    expect(commandQuery(value('请使用/goal')), const TextRange(start: 3, end: 8));
    expect(
      commandQuery(value('Use (/goal')),
      const TextRange(start: 5, end: 10),
    );
    expect(
      commandQuery(
        const TextEditingValue(
          text: 'Use /precise for this.',
          selection: TextSelection.collapsed(offset: 12),
        ),
      ),
      const TextRange(start: 4, end: 12),
    );
    for (final text in [
      'C:/work/file',
      '/tmp/file',
      'a/b',
      'See https://example.test',
      'Use tools/goal',
      '`/goal`',
      'Use `code /goal',
      'Use ``code /goal',
      'Use `code\n/goal',
      'Use @file',
      '```text\n/goal',
      '~~~\n/goal\n~~~',
      '    /goal',
    ]) {
      expect(commandQuery(value(text)), isNull, reason: text);
    }
    expect(
      commandQuery(
        value('/goal').copyWith(composing: const TextRange(start: 1, end: 5)),
      ),
      isNull,
    );
  });
  testWidgets(
    'sending moves skill references out of the composer with the submitted prompt',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge);
      await chat.selectSkills([reference]);
      chat.draft = 'Read this record';
      await chat.send();
      expect(bridge.submittedSkills, [reference]);
      expect(chat.draft, isEmpty);
      expect(chat.selectedSkills, isEmpty);
      expect(chat.pendingSkills, [reference]);
      expect(bridge.selections, isEmpty);
      await tester.pump();
      expect(find.text('Skills: precise-format'), findsOneWidget);
      expect(
        find.byKey(const Key('selected-skill-global-precise-format')),
        findsNothing,
      );
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 100));
      expect(chat.draft, 'Read this record');
      expect(chat.selectedSkills, [reference]);
      expect(bridge.selections, [reference]);
    },
  );
  testWidgets(
    'completion clears sent skills without consuming a newer selection',
    (tester) async {
      final bridge = CommandsBridge()..starts = 1;
      final chat = await mount(tester, bridge);
      await chat.selectSkills([reference]);
      chat.draft = 'Read this record';
      await chat.send();
      final newer = {...reference, 'name': 'goal'};
      await chat.selectSkills([newer]);
      chat.draft = 'A different follow-up';
      await tester.pump(const Duration(milliseconds: 100));
      await tester.pump();
      expect(chat.busy, isFalse);
      expect(chat.draft, 'A different follow-up');
      expect(chat.selectedSkills, [newer]);
      expect(bridge.selections, [newer]);
      expect(chat.pendingSkills, isEmpty);
      expect(find.text('Skills: precise-format'), findsOneWidget);
    },
  );
  testWidgets('a failed run cannot restore old references into a newer draft', (
    tester,
  ) async {
    final bridge = CommandsBridge();
    final chat = await mount(tester, bridge);
    await chat.selectSkills([reference]);
    chat.draft = 'Original request';
    await chat.send();
    final newer = {...reference, 'name': 'goal'};
    await chat.selectSkills([newer]);
    chat.draft = 'New request';
    await chat.stop();
    await tester.pump(const Duration(milliseconds: 100));
    expect(chat.draft, 'New request');
    expect(chat.selectedSkills, [newer]);
    expect(bridge.selections, [newer]);
  });
  testWidgets(
    'failed skill restoration stays visible and retries persistence before sending',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge);
      await chat.selectSkills([reference]);
      chat.draft = 'Read this record';
      await chat.send();
      bridge.failSelection = true;
      await chat.stop();
      await tester.pump(const Duration(milliseconds: 100));
      expect(chat.selectedSkills, [reference]);
      expect(chat.draft, 'Read this record');
      await chat.send();
      expect(bridge.starts, 1);
      expect(chat.draft, 'Read this record');
      bridge.failSelection = false;
      await chat.send();
      expect(bridge.starts, 2);
      expect(bridge.submittedSkills, [reference]);
      await tester.pump(const Duration(milliseconds: 100));
    },
  );
  testWidgets(
    'queue acceptance consumes selected skills while failed enqueue retains them',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge, busy: true);
      await chat.selectSkills([reference]);
      chat.draft = 'Queued instruction';
      bridge.failSave = true;
      await chat.queueDraft();
      expect(chat.draft, 'Queued instruction');
      expect(chat.selectedSkills, [reference]);
      bridge.failSave = false;
      await chat.queueDraft();
      expect(chat.draft, isEmpty);
      expect(chat.selectedSkills, isEmpty);
      expect(bridge.selections, isEmpty);
    },
  );
  testWidgets(
    'inline skill completion retains surrounding text and skips host commands',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge, busy: true);
      const draft = 'Please use /precise for this export.';
      await tester.enterText(find.byKey(const Key('composer-field-0')), draft);
      final input = tester
          .widget<RichComposer>(find.byKey(const Key('composer')))
          .controller;
      input.value = input.value.copyWith(
        selection: TextSelection.collapsed(offset: draft.indexOf(' for')),
      );
      await tester.pump();
      expect(find.text('Skills'), findsOneWidget);
      expect(find.byKey(const Key('command-command-stop')), findsNothing);
      await tester.sendKeyEvent(LogicalKeyboardKey.tab);
      await tester.pumpAndSettle();
      expect(chat.draft, 'Please use /precise-format for this export.');
      expect(input.selection.extentOffset, chat.draft.indexOf(' for'));
      expect(bridge.selections, [reference]);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Then use /goal',
      );
      await tester.pump();
      expect(find.byKey(const Key('command-command-goal')), findsNothing);
      await tester.tap(find.byKey(const Key('command-skill-goal')));
      await tester.pumpAndSettle();
      expect(chat.draft, 'Then use /goal ');
      expect(chat.selectedSkills.map((s) => s['name']), [
        'precise-format',
        'goal',
      ]);
      expect(chat.busy, isTrue);
      expect(bridge.starts, 0);
    },
  );
  testWidgets(
    'inline save failure retries and delayed saves preserve new typing',
    (tester) async {
      final bridge = CommandsBridge()..failSelection = true;
      final chat = await mount(tester, bridge);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Use /precise',
      );
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(chat.draft, 'Use /precise');
      expect(chat.selectedSkills, isEmpty);
      bridge.failSelection = false;
      bridge.selectionGate = Completer<void>();
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Use /precise and retain this typing',
      );
      bridge.selectionGate!.complete();
      await tester.pumpAndSettle();
      expect(chat.draft, 'Use /precise and retain this typing');
      expect(chat.selectedSkills, [reference]);
      expect(bridge.starts, 0);
    },
  );
  testWidgets(
    'compact rows retain insets and keyboard navigation reaches the last entry',
    (tester) async {
      await mount(tester, CommandsBridge(), busy: true);
      await tester.enterText(find.byKey(const Key('composer-field-0')), '/');
      await tester.pump();
      final panel = tester.getRect(find.byKey(const Key('command-picker')));
      final icon = tester.getRect(
        find.descendant(
          of: find.byKey(const Key('command-command-stop')),
          matching: find.byType(Icon),
        ),
      );
      expect(icon.left - panel.left, greaterThanOrEqualTo(16));
      expect(
        tester.getSize(find.byKey(const Key('command-command-stop'))).height,
        lessThan(60),
      );
      for (var i = 0; i < 3; i++) {
        await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
        await tester.pump();
      }
      expect(
        find.byKey(const Key('command-skill-goal')).hitTestable(),
        findsOneWidget,
      );
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(
        find.byKey(const Key('selected-skill-global-goal')),
        findsOneWidget,
      );
    },
  );
  testWidgets(
    'keyboard selection saves an exact reference and inspectable chip without sending text',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Please help\n/precise',
      );
      await tester.pump();
      expect(
        find.byKey(const Key('command-skill-precise-format')),
        findsOneWidget,
      );
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(bridge.selections, [reference]);
      expect(chat.draft, 'Please help\n');
      expect(bridge.starts, 0);
      await tester.tap(find.byTooltip('View skill'));
      await tester.pumpAndSettle();
      expect(find.textContaining('EXACT_PUBLIC_SNAPSHOT'), findsOneWidget);
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('Remove precise-format'));
      await tester.pumpAndSettle();
      expect(chat.selectedSkills, isEmpty);
      expect(chat.draft, 'Please help\n');
    },
  );
  testWidgets(
    'package reference keeps its archive identity and stale inspection never substitutes another package',
    (tester) async {
      final first = List.filled(64, 'a').join();
      final second = List.filled(64, 'b').join();
      final bridge = CommandsBridge()..packageId = first;
      final chat = await mount(tester, bridge);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        'Keep my thought\n/precise',
      );
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(bridge.selections.single['packageId'], first);
      bridge.packageId = second;
      await chat.refreshCommands();
      await tester.pumpAndSettle();
      await tester.tap(find.byTooltip('View skill'));
      await tester.pumpAndSettle();
      expect(find.textContaining('This selection has changed'), findsOneWidget);
      expect(find.textContaining('EXACT_PUBLIC_SNAPSHOT'), findsNothing);
      expect(chat.draft, 'Keep my thought\n');
      expect(bridge.starts, 0);
    },
  );
  testWidgets(
    'slash Stop uses the existing owner and pauses saved follow-ups without sending the command',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge);
      chat.draft = 'Original task';
      await chat.send();
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        '/stop',
      );
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump(const Duration(milliseconds: 50));
      await tester.pumpAndSettle();
      expect(bridge.starts, 1);
      expect(bridge.state['suspended'], isTrue);
      expect(chat.busy, isFalse);
      expect(chat.draft, 'Original task');
    },
  );
  testWidgets(
    'failed save retains slash text and selected value until explicit retry',
    (tester) async {
      final bridge = CommandsBridge()..failSelection = true;
      final chat = await mount(tester, bridge);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        '/precise',
      );
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(chat.draft, '/precise');
      expect(chat.selectedSkills, isEmpty);
      expect(chat.error, contains('draft remains'));
      bridge.failSelection = false;
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(chat.selectedSkills, [reference]);
      expect(chat.draft, isEmpty);
      expect(bridge.starts, 0);
    },
  );
  testWidgets(
    'escape keeps text and Stop; IME Enter never selects or submits',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge, busy: true);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        '/precise',
      );
      await tester.pump();
      final input = tester
          .widget<RichComposer>(find.byKey(const Key('composer')))
          .controller;
      final before = input.value;
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pump();
      expect(input.value, before);
      expect(chat.busy, isTrue);
      expect(find.byKey(const Key('command-picker')), findsNothing);
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: '/goal',
          selection: TextSelection.collapsed(offset: 5),
          composing: TextRange(start: 1, end: 5),
        ),
      );
      await tester.pump();
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();
      expect(bridge.selections, isEmpty);
      expect(bridge.starts, 0);
      expect(chat.inputEntries, isEmpty);
    },
  );
  testWidgets(
    'a skill sharing a command name cannot execute the unavailable host handler',
    (tester) async {
      final bridge = CommandsBridge();
      final chat = await mount(tester, bridge);
      await tester.enterText(
        find.byKey(const Key('composer-field-0')),
        '/goal',
      );
      await tester.pump();
      expect(find.text('Not available yet'), findsOneWidget);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pump();
      expect(chat.selectedSkills, isEmpty);
      expect(chat.draft, '/goal');
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();
      expect(chat.selectedSkills.single['name'], 'goal');
      expect(bridge.starts, 0);
    },
  );
  for (final dark in [false, true]) {
    testWidgets(
      'submitted skills stay visible with the saved message ${dark ? 'dark' : 'light'}',
      (tester) async {
        tester.view.physicalSize = const Size(540, 900);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final key = GlobalKey();
        final chat = await mount(
          tester,
          CommandsBridge()..starts = 1,
          dark: dark,
          boundary: key,
        );
        await chat.selectSkills([reference]);
        chat.draft = 'Read this record';
        await chat.send();
        await tester.pump();
        expect(find.text('Skills: precise-format'), findsOneWidget);
        expect(find.byTooltip('Remove precise-format'), findsNothing);
        await tester.pump(const Duration(milliseconds: 100));
        await tester.pumpAndSettle();
        expect(chat.busy, isFalse);
        expect(find.text('Skills: precise-format'), findsOneWidget);
        expect(tester.takeException(), isNull);
        final render =
            key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        await tester.runAsync(() async {
          final image = await render.toImage(pixelRatio: 1);
          final data = await image.toByteData(format: ui.ImageByteFormat.png);
          image.dispose();
          File(
            '../../output/harness/sent-skills-${dark ? 'dark' : 'light'}.png',
          ).writeAsBytesSync(data!.buffer.asUint8List());
        });
      },
    );
    testWidgets(
      'compact picker with retained selection ${dark ? 'dark' : 'light'}',
      (tester) async {
        tester.view.physicalSize = const Size(540, 900);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final bridge = CommandsBridge()..selections = [reference];
        final key = GlobalKey();
        await mount(tester, bridge, busy: true, dark: dark, boundary: key);
        await tester.enterText(find.byKey(const Key('composer-field-0')), '/');
        await tester.pumpAndSettle();
        expect(find.byTooltip('Stop response'), findsOneWidget);
        expect(tester.takeException(), isNull);
        final render =
            key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        await tester.runAsync(() async {
          final image = await render.toImage(pixelRatio: 1);
          final data = await image.toByteData(format: ui.ImageByteFormat.png);
          image.dispose();
          final file = File(
            '../../output/harness/eh02-${dark ? 'dark' : 'light'}-compact.png',
          );
          file.parent.createSync(recursive: true);
          file.writeAsBytesSync(data!.buffer.asUint8List());
        });
      },
    );
    testWidgets(
      'wide inline picker with long descriptions ${dark ? 'dark' : 'light'}',
      (tester) async {
        tester.view.physicalSize = const Size(1200, 900);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final key = GlobalKey();
        final chat = await mount(
          tester,
          CommandsBridge(),
          dark: dark,
          boundary: key,
        );
        const description =
            'Structure exports with consistent headings, useful summaries and detailed references for the requested audience, retaining all relevant supporting evidence.';
        (chat.commandCatalogue['skills'] as List).first['description'] =
            description;
        await tester.enterText(
          find.byKey(const Key('composer-field-0')),
          'Please use /precise',
        );
        await tester.pumpAndSettle();
        expect(find.text('Skills'), findsOneWidget);
        expect(tester.takeException(), isNull);
        final render =
            key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
        await tester.runAsync(() async {
          final image = await render.toImage(pixelRatio: 1);
          final data = await image.toByteData(format: ui.ImageByteFormat.png);
          image.dispose();
          final file = File(
            '../../output/harness/inline-skills-${dark ? 'dark' : 'light'}-wide.png',
          );
          file.parent.createSync(recursive: true);
          file.writeAsBytesSync(data!.buffer.asUint8List());
        });
      },
    );
  }
  testWidgets('compact picker accommodates larger text and reduced motion', (
    tester,
  ) async {
    tester.view.physicalSize = const Size(540, 900);
    tester.view.devicePixelRatio = 1;
    tester.platformDispatcher.textScaleFactorTestValue = 2;
    tester.platformDispatcher.accessibilityFeaturesTestValue =
        FakeAccessibilityFeatures(disableAnimations: true);
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    addTearDown(tester.platformDispatcher.clearTextScaleFactorTestValue);
    addTearDown(tester.platformDispatcher.clearAccessibilityFeaturesTestValue);
    final key = GlobalKey();
    await mount(tester, CommandsBridge(), boundary: key);
    await tester.enterText(find.byKey(const Key('composer-field-0')), 'Use /');
    await tester.pumpAndSettle();
    expect(tester.takeException(), isNull);
    expect(
      tester
          .getSize(find.byKey(const Key('command-skill-precise-format')))
          .height,
      greaterThan(80),
    );
    final render =
        key.currentContext!.findRenderObject()! as RenderRepaintBoundary;
    await tester.runAsync(() async {
      final image = await render.toImage(pixelRatio: 1);
      final data = await image.toByteData(format: ui.ImageByteFormat.png);
      image.dispose();
      File('../../output/harness/inline-skills-large-text.png')
          .writeAsBytesSync(data!.buffer.asUint8List());
    });
  });
}
