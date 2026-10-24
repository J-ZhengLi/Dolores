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
  String? packageId;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
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
      if (failSelection) throw StateError('Storage is locked');
      selections = List<Map<String, dynamic>>.from(
        command['selections'] as List,
      );
      return selections;
    }
    return super.call(command);
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
    for (final text in [
      'C:/work/file',
      '/tmp/file',
      'a/b',
      'Use /goal',
      '`/goal`',
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
      await tester.tap(find.byTooltip('Inspect reviewed skill'));
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
      await tester.tap(find.byTooltip('Inspect reviewed skill'));
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
  }
}
