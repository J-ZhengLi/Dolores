import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/models.dart';
import 'package:dolores_flutter/settings.dart';

import 'models_test.dart' show DetailsBridge;
import 'settings_test.dart' show HarnessSettingsBridge;

void main() {
  for (final dark in [false, true]) {
    testWidgets('Advanced overview fits compact theme $dark', (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(HarnessSettingsBridge())..loading = false;
      await tester.pumpWidget(
        MaterialApp(
          theme: ThemeData(
            brightness: dark ? Brightness.dark : Brightness.light,
          ),
          home: SettingsWindow(
            chat: chat,
            initial: SettingsCategory.advancedHome,
          ),
        ),
      );
      await tester.pumpAndSettle();
      expect(find.text('Task limits'), findsOneWidget);
      expect(find.text('Skill testing'), findsNothing);
      await tester.scrollUntilVisible(
        find.text('Skill testing & learning'),
        100,
        scrollable: find
            .descendant(
              of: find.byKey(const Key('settings-window')),
              matching: find.byType(Scrollable),
            )
            .last,
      );
      await tester.pumpAndSettle();
      await tester.tap(find.text('Skill testing & learning'));
      await tester.pumpAndSettle();
      expect(find.text('Skill testing'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    });
  }
  testWidgets('Escape closes a protected model route and guards edited text', (
    tester,
  ) async {
    final bridge = DetailsBridge();
    final chat = ChatController(bridge)..loading = false;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                barrierDismissible: false,
                builder: (_) =>
                    ModelDetailsEditor(chat: chat, model: 'fixture'),
              ),
              child: const Text('Open'),
            );
          },
        ),
      ),
    );
    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(find.byType(ModelDetailsEditor), findsNothing);
    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
    await tester.enterText(
      find.byKey(const Key('model-detail-context')),
      '8192',
    );
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(find.text('Unsaved changes'), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(find.text('Unsaved changes'), findsNothing);
    expect(
      tester
          .widget<TextField>(find.byKey(const Key('model-detail-context')))
          .controller!
          .text,
      '8192',
    );
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Discard'));
    await tester.pumpAndSettle();
    expect(find.byType(ModelDetailsEditor), findsNothing);
    expect(
      bridge.commands.where((c) => c['command'] == 'setModelDetails'),
      isEmpty,
    );
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });

  testWidgets('Escape closes the real Settings route without a focused field', (
    tester,
  ) async {
    final chat = ChatController(HarnessSettingsBridge())..loading = false;
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) {
            return TextButton(
              onPressed: () => showSettings(context, chat),
              child: const Text('Open'),
            );
          },
        ),
      ),
    );
    await tester.tap(find.text('Open'));
    await tester.pumpAndSettle();
    await tester.sendKeyEvent(LogicalKeyboardKey.escape);
    await tester.pumpAndSettle();
    expect(find.byType(SettingsWindow), findsNothing);
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });

  testWidgets(
    'Tools exposes Skills and Advanced starts with a short overview',
    (tester) async {
      final chat = ChatController(HarnessSettingsBridge())..loading = false;
      await tester.pumpWidget(MaterialApp(home: SettingsWindow(chat: chat)));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('settings-tools')));
      await tester.pumpAndSettle();
      expect(find.text('Import, create or learn from a chat'), findsOneWidget);
      await tester.tap(find.byKey(const Key('settings-advanced')));
      await tester.pumpAndSettle();
      expect(find.text('Tuning, testing and troubleshooting'), findsOneWidget);
      expect(find.text('Task limits'), findsWidgets);
      expect(find.text('Compare instructions'), findsNothing);
      await tester.tap(find.text('Diagnostics & storage'));
      await tester.pumpAndSettle();
      expect(find.text('Compare instructions'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
}
