import 'dart:async';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/desktop_frame.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/settings.dart';
import 'package:dolores_flutter/workspace_dock.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show DesignBridge, capture, viewport;
import 'support/render_fonts.dart';

void main() {
  setUp(() {
    debugDefaultTargetPlatformOverride = TargetPlatform.windows;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(const MethodChannel('window_manager'), (
          call,
        ) async {
          if (call.method == 'isMaximized') return false;
          return null;
        });
  });
  tearDown(() {
    debugDefaultTargetPlatformOverride = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(const MethodChannel('window_manager'), null);
  });

  for (final mode in [ThemeMode.light, ThemeMode.dark]) {
    testWidgets(
      'title Settings stays visible beside Minimize with the dock hidden ($mode)',
      (t) async {
        viewport(t, const Size(1100, 800));
        await t.runAsync(loadReviewFonts);
        final chat = ChatController(DesignBridge())
          ..loading = false
          ..configured = true
          ..draft = 'Keep this unfinished message';
        final host = AppHost(chat);
        final key = GlobalKey();
        await t.pumpWidget(
          DoloresApp(
            chat: chat,
            host: host,
            desktopFrame: true,
            themeMode: mode,
            captureKey: key,
          ),
        );
        await t.pumpAndSettle();
        await t.pump(const Duration(seconds: 3));
        await t.pumpAndSettle();
        expect(find.byKey(const Key('dock-reveal')), findsOneWidget);
        final gear = find.byKey(const Key('window-settings'));
        expect(gear.hitTestable(), findsOneWidget);
        expect(
          t.getRect(gear).right,
          t.getRect(find.byKey(const Key('window-minimize'))).left,
        );
        expect(
          find.descendant(
            of: gear,
            matching: find.byIcon(Icons.settings_outlined),
          ),
          findsOneWidget,
        );
        await capture(t, key, 'title-settings-${mode.name}');
        await t.tap(gear);
        await t.pumpAndSettle();
        expect(find.byType(SettingsWindow), findsOneWidget);
        expect(find.byKey(const Key('theme-light')), findsOneWidget);
        await t.tap(gear); // The frame is outside the modal's Navigator.
        await t.pumpAndSettle();
        expect(find.byType(SettingsWindow), findsOneWidget);
        await t.tap(find.byKey(const Key('close-settings')));
        await t.pumpAndSettle();
        expect(chat.draft, 'Keep this unfinished message');
        t.view.physicalSize = const Size(420, 640);
        await t.pumpAndSettle();
        expect(gear.hitTestable(), findsOneWidget);
        expect(
          t.getRect(gear).right,
          t.getRect(find.byKey(const Key('window-minimize'))).left,
        );
        await capture(t, key, 'title-settings-compact-${mode.name}');
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        host.dispose();
        debugDefaultTargetPlatformOverride = null;
      },
    );
  }

  testWidgets('hidden navigation announces the dock and its destinations', (
    t,
  ) async {
    final semantics = t.ensureSemantics();
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Align(
            alignment: Alignment.bottomCenter,
            child: WorkspaceDock(
              selected: 0,
              items: [
                DockItem(
                  key: const Key('home'),
                  label: 'Home',
                  icon: Icons.home,
                  onPressed: () {},
                ),
                DockItem(
                  key: const Key('settings'),
                  label: 'Settings',
                  icon: Icons.settings,
                  onPressed: () {},
                ),
              ],
            ),
          ),
        ),
      ),
    );
    await t.pump(const Duration(seconds: 3));
    await t.pumpAndSettle();
    expect(
      find.bySemanticsLabel(RegExp('Show navigation dock.*Settings')),
      findsOneWidget,
    );
    expect(find.byKey(const Key('settings')).hitTestable(), findsNothing);
    await t.tap(find.byKey(const Key('dock-reveal')));
    await t.pumpAndSettle();
    expect(find.byKey(const Key('settings')).hitTestable(), findsOneWidget);
    await t.pumpWidget(const SizedBox());
    semantics.dispose();
    debugDefaultTargetPlatformOverride = null;
  });

  testWidgets(
    'title Settings respects an open review and works by keyboard afterwards',
    (t) async {
      viewport(t, const Size(1000, 800));
      final chat = ChatController(DesignBridge())..loading = false;
      await t.pumpWidget(DoloresApp(chat: chat, desktopFrame: true));
      await t.pumpAndSettle();
      final navigator = GlobalObjectKey<NavigatorState>(chat).currentState!;
      unawaited(
        showDialog<void>(
          context: navigator.overlay!.context,
          builder: (_) => const AlertDialog(title: Text('Keep this review')),
        ),
      );
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('window-settings')));
      await t.pumpAndSettle();
      expect(find.text('Keep this review'), findsOneWidget);
      expect(find.byType(SettingsWindow), findsNothing);
      navigator.pop();
      await t.pumpAndSettle();
      final gear = find.byKey(const Key('window-settings'));
      Focus.of(
        t.element(find.descendant(of: gear, matching: find.byType(Icon)).first),
      ).requestFocus();
      await t.pump();
      await t.sendKeyEvent(LogicalKeyboardKey.enter);
      await t.pumpAndSettle();
      expect(find.byType(SettingsWindow), findsOneWidget);
      await t.tap(find.byKey(const Key('close-settings')));
      await t.pumpAndSettle();
      expect(
        Focus.of(
          t.element(
            find.descendant(of: gear, matching: find.byType(Icon)).first,
          ),
        ).hasFocus,
        isTrue,
      );
      await t.pumpWidget(const SizedBox());
      chat.dispose();
      debugDefaultTargetPlatformOverride = null;
    },
  );

  testWidgets(
    'failed Settings opening retains content and explicit retry is single-flight',
    (t) async {
      var attempts = 0;
      final pending = Completer<void>();
      await t.pumpWidget(
        MaterialApp(
          builder: (_, child) => DesktopFrame(
            onOpenSettings: () async {
              attempts++;
              if (attempts == 1) throw StateError('Fixture unavailable');
              await pending.future;
            },
            child: child!,
          ),
          home: const Scaffold(body: Text('Keep usable work')),
        ),
      );
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('window-settings')));
      await t.pumpAndSettle();
      expect(find.text('Could not open Settings.'), findsOneWidget);
      expect(find.text('Keep usable work'), findsOneWidget);
      await t.tap(find.text('Retry'));
      await t.pump();
      final button = t.widget<TextButton>(
        find.byKey(const Key('window-settings')),
      );
      expect(button.onPressed, isNull);
      expect(
        t
            .widget<TextButton>(find.byKey(const Key('window-minimize')))
            .onPressed,
        isNotNull,
      );
      expect(attempts, 2);
      pending.complete();
      await t.pumpAndSettle();
      expect(find.text('Could not open Settings.'), findsNothing);
      expect(
        t
            .widget<TextButton>(find.byKey(const Key('window-settings')))
            .onPressed,
        isNotNull,
      );
      await t.pumpWidget(const SizedBox());
      debugDefaultTargetPlatformOverride = null;
    },
  );
}
