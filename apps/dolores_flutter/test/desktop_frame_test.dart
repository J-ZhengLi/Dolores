import 'dart:async';

import 'package:dolores_flutter/desktop_frame.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:window_manager/window_manager.dart';

class LocalBridge implements ChatBridge {
  @override
  Future<dynamic> call(Map<String, dynamic> command) async => null;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  const channel = MethodChannel('window_manager');
  final calls = <MethodCall>[];
  bool maximized = false, failMaximize = false, failInitialize = false;
  Completer<void>? pendingMaximize;
  setUp(() {
    calls.clear();
    maximized = failMaximize = failInitialize = false;
    pendingMaximize = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, (call) async {
          calls.add(call);
          if (call.method == 'ensureInitialized' && failInitialize) {
            throw PlatformException(code: 'fixture');
          }
          if (call.method == 'isMaximized') return maximized;
          if (call.method == 'isFullScreen' || call.method == 'isMinimized') {
            return false;
          }
          if (call.method == 'maximize') {
            if (failMaximize) throw PlatformException(code: 'fixture');
            await pendingMaximize?.future;
            maximized = true;
          }
          if (call.method == 'unmaximize') maximized = false;
          return null;
        });
  });
  tearDown(() {
    debugDefaultTargetPlatformOverride = null;
    TestDefaultBinaryMessengerBinding.instance.defaultBinaryMessenger
        .setMockMethodCallHandler(channel, null);
  });

  Future<void> mount(WidgetTester tester, {bool dark = false}) async {
    debugDefaultTargetPlatformOverride ??= TargetPlatform.windows;
    await tester.pumpWidget(
      MaterialApp(
        theme: doloresTheme(dark),
        builder: (_, child) => DesktopFrame(child: child!),
        home: const Scaffold(body: TextField(key: Key('draft'))),
      ),
    );
    await tester.pumpAndSettle();
  }

  testWidgets(
    'The real app keeps its draft and window controls through theme and dialog changes',
    (tester) async {
      debugDefaultTargetPlatformOverride = TargetPlatform.windows;
      tester.view.physicalSize = const Size(1000, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final chat = ChatController(LocalBridge())..loading = false;
      chat.appearance = 'light';
      chat.draft = 'unfinished draft';
      await tester.pumpWidget(DoloresApp(chat: chat, desktopFrame: true));
      await tester.pumpAndSettle();
      expect(find.text('unfinished draft'), findsOneWidget);
      expect(find.text('Dolores'), findsOneWidget);
      chat.appearance = 'dark';
      await tester.pumpAndSettle();
      expect(find.text('unfinished draft'), findsOneWidget);
      await tester.tap(find.byKey(const Key('settings')));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('window-minimize')), findsOneWidget);
      await tester.tap(find.byKey(const Key('window-maximize')));
      await tester.pumpAndSettle();
      expect(find.byTooltip('Restore'), findsOneWidget);
      expect(find.text('unfinished draft'), findsOneWidget);
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
      debugDefaultTargetPlatformOverride = null;
    },
  );

  testWidgets(
    'The title strip follows light and dark themes without branding',
    (tester) async {
      for (final dark in [false, true]) {
        await mount(tester, dark: dark);
        expect(
          tester
              .widget<Material>(
                find
                    .ancestor(
                      of: find.byKey(const Key('desktop-title-bar')),
                      matching: find.byType(Material),
                    )
                    .first,
              )
              .color,
          Palette(dark).bg,
        );
        expect(find.text('Dolores'), findsNothing);
        expect(
          find.descendant(
            of: find.byKey(const Key('window-drag-area')),
            matching: find.byWidgetPredicate(
              (widget) => widget is Container && widget.color != null,
            ),
          ),
          findsNothing,
        );
        expect(find.byTooltip('Minimize'), findsOneWidget);
        expect(find.byTooltip('Maximize'), findsOneWidget);
        expect(find.byTooltip('Close'), findsOneWidget);
      }
      await tester.pumpWidget(const SizedBox());
      expect(windowManager.hasListeners, isFalse);
      debugDefaultTargetPlatformOverride = null;
    },
  );

  testWidgets('OS maximize events and double click switch to Restore', (
    tester,
  ) async {
    await mount(tester);
    maximized = true;
    for (final listener in windowManager.listeners) {
      listener.onWindowMaximize();
    }
    await tester.pump();
    expect(find.byTooltip('Restore'), findsOneWidget);
    await tester.tap(find.byKey(const Key('window-restore')));
    await tester.pumpAndSettle();
    expect(calls.where((c) => c.method == 'unmaximize'), hasLength(1));
    expect(find.byTooltip('Maximize'), findsOneWidget);
    final drag = find.byKey(const Key('window-drag-area'));
    await tester.drag(drag, const Offset(100, 50));
    await tester.pumpAndSettle();
    expect(calls.where((c) => c.method == 'startDragging'), hasLength(1));
    await tester.tap(drag);
    await tester.pump(const Duration(milliseconds: 50));
    await tester.tap(drag);
    await tester.pumpAndSettle();
    expect(maximized, isTrue);
    expect(find.byTooltip('Restore'), findsOneWidget);
    await tester.tap(find.byKey(const Key('window-minimize')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('window-close')));
    await tester.pumpAndSettle();
    expect(calls.where((c) => c.method == 'minimize'), hasLength(1));
    expect(calls.where((c) => c.method == 'close'), hasLength(1));
    debugDefaultTargetPlatformOverride = null;
  });

  testWidgets(
    'Failed and pending window actions preserve edits and allow explicit retry',
    (tester) async {
      await mount(tester);
      await tester.enterText(find.byKey(const Key('draft')), 'keep this draft');
      failMaximize = true;
      await tester.tap(find.byKey(const Key('window-maximize')));
      await tester.pumpAndSettle();
      expect(find.text('Could not resize the window.'), findsOneWidget);
      expect(find.text('keep this draft'), findsOneWidget);
      failMaximize = false;
      pendingMaximize = Completer<void>();
      await tester.tap(find.text('Retry'));
      await tester.pump();
      await tester.tap(find.byKey(const Key('window-maximize')));
      await tester.pump();
      expect(calls.where((c) => c.method == 'maximize'), hasLength(2));
      pendingMaximize!.complete();
      await tester.pumpAndSettle();
      expect(find.text('Could not resize the window.'), findsNothing);
      expect(find.text('keep this draft'), findsOneWidget);
      expect(find.byTooltip('Restore'), findsOneWidget);
      debugDefaultTargetPlatformOverride = null;
    },
  );

  testWidgets(
    'Compact windows fit controls and macOS reserves native traffic lights',
    (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await mount(tester);
      expect(tester.takeException(), isNull);
      debugDefaultTargetPlatformOverride = null;
      expect(tester.getRect(find.byKey(const Key('window-close'))).right, 420);
      await tester.pumpWidget(const SizedBox());
      debugDefaultTargetPlatformOverride = TargetPlatform.macOS;
      await mount(tester);
      expect(find.byTooltip('Close'), findsNothing);
      expect(
        tester.getRect(find.byKey(const Key('window-drag-area'))).left,
        80,
      );
      expect(tester.takeException(), isNull);
      debugDefaultTargetPlatformOverride = null;
    },
  );

  test(
    'Initialize hides native branding; failure restores the native frame',
    () async {
      debugDefaultTargetPlatformOverride = TargetPlatform.windows;
      expect(await initializeDesktopFrame(), isTrue);
      final style = calls.singleWhere((c) => c.method == 'setTitleBarStyle');
      expect(style.arguments['titleBarStyle'], 'hidden');
      expect(style.arguments['windowButtonVisibility'], isFalse);
      calls.clear();
      failInitialize = true;
      expect(await initializeDesktopFrame(), isFalse);
      expect(calls.last.method, 'setTitleBarStyle');
      expect(calls.last.arguments['titleBarStyle'], 'normal');
      debugDefaultTargetPlatformOverride = null;
    },
  );
}
