import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/settings.dart';
import 'package:dolores_flutter/settings_frame.dart';

import 'widget_test.dart' show ConnectionBridge;
import 'web_settings_test.dart' show WebBridge;
import 'dolores_settings_test.dart' show SettingsBridge;
import 'memory_test.dart' show MemoryBridge;

class HarnessSettingsBridge extends ConnectionBridge {
  final web = WebBridge();
  final scoped = SettingsBridge();
  final memory = MemoryBridge();
  String theme = 'system';
  Completer<void>? hold;
  bool failTheme = false;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    switch (command['command']) {
      case 'saveAppearance':
        commands.add(command);
        if (hold != null) await hold!.future;
        if (failTheme) throw StateError('Could not save theme.');
        theme = command['theme'] as String;
        return theme;
      case 'bootstrap':
        return {...await super.call(command) as Map, 'appearance': theme};
      case 'webSettings':
      case 'saveWebSettings':
        return web.call(command);
      case 'scopedSettings':
      case 'saveScopedSettings':
        return scoped.call(command);
      case 'memories':
      case 'saveMemory':
        return memory.call(command);
    }
    return super.call(command);
  }
}

void size(WidgetTester tester, Size value) {
  tester.view.physicalSize = value;
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
}

Future<void> category(WidgetTester tester, String name, String label) async {
  await tester.enterText(
    find.byKey(const Key('settings-search')),
    label == 'Appearance' ? 'Theme' : label,
  );
  await tester.pumpAndSettle();
  await tester.tap(find.byKey(Key('setting-result-$name-connection')));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'Six sections fit 420x480 and search keeps cached drafts on failed close',
    (tester) async {
      size(tester, const Size(420, 480));
      final bridge = HarnessSettingsBridge()..web.stale = true;
      final chat = ChatController(bridge)..loading = false;
      await tester.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => TextButton(
              onPressed: () => showSettings(context, chat),
              child: const Text('Open'),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<DropdownButton<SettingsSection>>(
              find.byWidgetPredicate(
                (w) => w is DropdownButton<SettingsSection>,
              ),
            )
            .items!
            .length,
        6,
      );
      expect(find.text('General'), findsWidgets);
      await category(tester, 'web', 'Web search');
      final toggle = find.descendant(
        of: find.byKey(const Key('web-enabled')),
        matching: find.byType(Switch),
      );
      await tester.ensureVisible(toggle);
      await tester.pumpAndSettle();
      await tester.tap(toggle);
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('close-settings')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Save').last);
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('settings-window')), findsOneWidget);
      expect(
        tester
            .widget<SwitchListTile>(find.byKey(const Key('web-enabled')))
            .value,
        false,
      );
      expect(find.textContaining('retained draft'), findsOneWidget);
      await tester.tap(find.byKey(const Key('close-settings')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Discard'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('settings-window')), findsNothing);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets(
    'One entry, retained edits across categories, and explicit saves',
    (tester) async {
      size(tester, const Size(1120, 780));
      final bridge = HarnessSettingsBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..draft = 'Unsent work';
      await tester.pumpWidget(DoloresApp(chat: chat));
      expect(find.byKey(const Key('settings')), findsOneWidget);
      expect(find.byKey(const Key('connection')), findsNothing);
      expect(find.byKey(const Key('request-settings')), findsNothing);
      expect(find.byKey(const Key('memory')), findsNothing);
      await tester.tap(find.byKey(const Key('settings')));
      await tester.pumpAndSettle();
      await category(tester, 'models', 'Models');
      await tester.enterText(
        find.byKey(const Key('base-url')),
        'https://draft.example/v1',
      );
      await category(tester, 'web', 'Web search');
      await tester.tap(find.text('Search connection'));
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('web-provider')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Custom SearXNG').last);
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('web-endpoint')),
        'https://search.example/search',
      );
      await category(tester, 'models', 'Models');
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('base-url')))
            .controller!
            .text,
        'https://draft.example/v1',
      );
      await category(tester, 'web', 'Web search');
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('web-endpoint')))
            .controller!
            .text,
        'https://search.example/search',
      );
      await category(tester, 'memory', 'Memory');
      expect(find.byKey(const Key('memory-help')), findsOneWidget);
      expect(find.byType(Dialog), findsOneWidget);
      await tester.tap(find.byKey(const Key('close-settings')));
      await tester.pumpAndSettle();
      expect(find.text('Unsaved changes'), findsOneWidget);
      await tester.tap(find.text('Keep editing'));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('settings-window')), findsOneWidget);
      await tester.tap(find.byKey(const Key('close-settings')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Discard'));
      await tester.pumpAndSettle();
      expect(chat.draft, 'Unsent work');
      expect(
        bridge.commands.where((c) => c['command'] == 'configure'),
        isEmpty,
      );
      expect(
        bridge.web.calls.where((c) => c['command'] == 'saveWebSettings'),
        isEmpty,
      );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  for (final compact in [false, true]) {
    testWidgets(
      'Theme updates whole app and pending/failed saves recover ($compact)',
      (tester) async {
        size(tester, compact ? const Size(420, 640) : const Size(1120, 780));
        final bridge = HarnessSettingsBridge();
        final chat = ChatController(bridge)..loading = false;
        await tester.pumpWidget(DoloresApp(chat: chat));
        if (compact) {
          await tester.tap(find.byTooltip('Conversations'));
          await tester.pumpAndSettle();
        }
        await tester.tap(find.byKey(const Key('settings')));
        await tester.pumpAndSettle();
        await category(tester, 'appearance', 'Appearance');
        bridge.hold = Completer();
        bridge.failTheme = true;
        await tester.tap(find.byKey(const Key('theme-dark')));
        await tester.pump();
        await tester.pump();
        expect(
          tester
              .widget<TextButton>(find.byKey(const Key('close-settings')))
              .onPressed,
          isNull,
        );
        await tester.sendKeyEvent(LogicalKeyboardKey.escape);
        await tester.pump();
        expect(find.byType(SettingsWindow), findsOneWidget);
        bridge.hold!.complete();
        await tester.pumpAndSettle();
        expect(chat.appearance, 'system');
        expect(
          find.textContaining('Your previous theme is retained'),
          findsOneWidget,
        );
        bridge.hold = null;
        bridge.failTheme = false;
        await tester.ensureVisible(find.byKey(const Key('theme-dark')));
        await tester.tap(find.byKey(const Key('theme-dark')));
        await tester.pumpAndSettle();
        expect(chat.appearance, 'dark');
        expect(
          Theme.of(tester.element(find.byKey(const Key('theme-dark'))))
              .brightness,
          Brightness.dark,
        );
        await tester.ensureVisible(find.byKey(const Key('theme-light')));
        await tester.tap(find.byKey(const Key('theme-light')));
        await tester.pumpAndSettle();
        expect(
          Theme.of(tester.element(find.byKey(const Key('theme-light'))))
              .brightness,
          Brightness.light,
        );
        await tester.tap(find.byKey(const Key('close-settings')));
        await tester.pumpAndSettle();
        expect(
          Theme.of(tester.element(find.byKey(const Key('composer-frame'))))
              .brightness,
          Brightness.light,
        );
        final restarted = ChatController(bridge);
        await restarted.initialize();
        expect(restarted.appearance, 'light');
        restarted.dispose();
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
  testWidgets('Personalization reset preserves model and task overrides', (
    tester,
  ) async {
    size(tester, const Size(1120, 780));
    final bridge = HarnessSettingsBridge();
    bridge.scoped.patch['task'] = {
      'modelCalls': 6,
      'toolCalls': 5,
      'segments': 3,
    };
    final chat = ChatController(bridge)
      ..loading = false
      ..session = 'chat';
    await tester.pumpWidget(
      MaterialApp(
        home: SettingsWindow(
          chat: chat,
          initial: SettingsCategory.personalization,
        ),
      ),
    );
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('scoped-output')), findsNothing);
    expect(find.text('All chats'), findsOneWidget);
    expect(find.text('Keep explanations brief'), findsOneWidget);
    await tester.tap(find.byKey(const Key('settings-scope')));
    await tester.pumpAndSettle();
    await tester.tap(find.text('This chat').last);
    await tester.pumpAndSettle();
    await tester.tap(find.text('Use inherited settings'));
    await tester.pumpAndSettle();
    expect(bridge.scoped.patch['interaction'], isNull);
    expect(bridge.scoped.patch['generation']['maxOutputTokens'], 256);
    expect(bridge.scoped.patch['task']['modelCalls'], 6);
    await tester.enterText(
      find.byKey(const Key('settings-search')),
      'Scope overrides',
    );
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('setting-result-models-overrides')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('scoped-output')), findsOneWidget);
    expect(find.text('Override interaction for this scope'), findsNothing);
    expect(tester.takeException(), isNull);
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });
  testWidgets(
    'Changed endpoint refuses a retained response draft and provides reload',
    (tester) async {
      size(tester, const Size(1120, 780));
      final bridge = HarnessSettingsBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..model = 'fixture'
        ..enabledModels = ['fixture'];
      await tester.pumpWidget(
        MaterialApp(
          home: SettingsWindow(chat: chat, modelsPage: ModelsPage.defaults),
        ),
      );
      await tester.pumpAndSettle();
      await tester.enterText(
        find.byKey(const Key('output-token-limit')),
        '4096',
      );
      chat.baseUrl = 'https://other.example/v1';
      chat.enabledModels = ['replacement-model'];
      chat.model = 'replacement-model';
      await tester.tap(find.byKey(const Key('save-request-settings')));
      await tester.pumpAndSettle();
      await tester.ensureVisible(
        find.textContaining('The model connection changed'),
      );
      expect(
        find.textContaining('The model connection changed'),
        findsOneWidget,
      );
      expect(
        tester
            .widget<TextFormField>(find.byKey(const Key('output-token-limit')))
            .controller!
            .text,
        '4096',
      );
      expect(
        bridge.commands.where((c) => c['command'] == 'setModelRequestSettings'),
        isEmpty,
      );
      await tester.tap(find.text('Reload response settings'));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<TextFormField>(find.byKey(const Key('output-token-limit')))
            .controller!
            .text,
        '',
      );
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );
  testWidgets('Conversation updates do not rebuild the application theme', (
    tester,
  ) async {
    final chat = ChatController(HarnessSettingsBridge())..loading = false;
    await tester.pumpWidget(DoloresApp(chat: chat));
    final before = tester.widget<MaterialApp>(find.byType(MaterialApp));
    chat.invalidateContext();
    await tester.pump();
    expect(
      identical(before, tester.widget<MaterialApp>(find.byType(MaterialApp))),
      isTrue,
    );
    await chat.saveAppearance('dark');
    await tester.pumpAndSettle();
    expect(
      tester.widget<MaterialApp>(find.byType(MaterialApp)).themeMode,
      ThemeMode.dark,
    );
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });
  testWidgets('Secondary editor retains a separate dialog route', (
    tester,
  ) async {
    final lock = ValueNotifier(false);
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => SettingsEmbedding(
            route: ModalRoute.of(context),
            pending: lock,
            child: TextButton(
              onPressed: () => showDialog<void>(
                context: context,
                builder: (dialogContext) => Text(
                  SettingsEmbedding.of(dialogContext) == null
                      ? 'Separate dialog'
                      : 'Incorrect embedding',
                ),
              ),
              child: const Text('Open detail'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('Open detail'));
    await tester.pumpAndSettle();
    expect(find.text('Separate dialog'), findsOneWidget);
    await tester.pumpWidget(const SizedBox());
    lock.dispose();
  });
}
