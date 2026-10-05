import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/desktop_settings.dart';

class DesktopBridge implements ChatBridge {
  final calls = <String>[];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    calls.add(value['command'] as String);
    switch (value['command']) {
      case 'desktopState':
        return {
          'available': true,
          'captures': [],
          'models': [],
          'adapter': 'Selected window only',
          'bounds': 'Bounded capture',
          'sharing': 'Local until explicitly shared',
        };
      case 'desktopObserve':
        return {'id': value['id']};
      case 'poll':
        return [
          {
            'type': 'done',
            'error': 'Window closed. Refresh and capture again.',
          },
        ];
    }
    throw StateError('Unexpected request ${value['command']}');
  }
}

class InputDesktopBridge extends DesktopBridge {
  static final capture = {
    'id': 'fixture-capture',
    'createdAt': 1,
    'observation': {
      'target': {'title': 'Local form', 'handle': 17},
      'width': 1,
      'height': 1,
      'dpi': 96,
    },
  };
  @override
  Future<dynamic> call(Map<String, dynamic> value) async {
    calls.add(value['command'] as String);
    switch (value['command']) {
      case 'desktopState':
        return {
          'available': true,
          'captures': [capture],
          'models': ['fixture-vision'],
          'adapter': 'Selected window',
          'bounds': 'Bounded',
          'sharing': 'Explicit only',
          'access': null,
        };
      case 'desktopPreview':
        return {
          'capture': capture,
          'data': base64Encode(
            base64Decode(
              'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+/lYQAAAAASUVORK5CYII=',
            ),
          ),
        };
      case 'desktopGrant':
        throw StateError('Window moved. Capture again; no input dispatched.');
    }
    throw StateError('Unexpected command');
  }
}

void main() {
  testWidgets(
    'desktop consent stays explicit and failed grants retain draft and preview in compact themes',
    (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      for (final brightness in Brightness.values) {
        await tester.pumpWidget(const SizedBox.shrink());
        final bridge = InputDesktopBridge(),
            chat = ChatController(InputDesktopBridge());
        final local = ChatController(bridge)
          ..session = 'one'
          ..workspaceRoot = 'fixture'
          ..draft = 'Keep my goal';
        addTearDown(chat.dispose);
        addTearDown(local.dispose);
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(brightness: brightness),
            home: DesktopSettingsInspector(chat: local),
          ),
        );
        await tester.pumpAndSettle();
        final snapshots = find.byWidgetPredicate(
          (w) =>
              w is DropdownButtonFormField<String> &&
              w.decoration.labelText == 'Saved screenshots in this chat',
        );
        await tester.scrollUntilVisible(snapshots, 180);
        await Scrollable.ensureVisible(
          tester.element(snapshots),
          alignment: 0.5,
        );
        await tester.pumpAndSettle();
        await tester.tap(snapshots);
        await tester.pumpAndSettle();
        await tester.tap(find.textContaining('Local form ·').last);
        await tester.pumpAndSettle();
        await tester.scrollUntilVisible(
          find.text('Allow input to this captured window'),
          180,
        );
        await Scrollable.ensureVisible(
          tester.element(find.text('Allow input to this captured window')),
          alignment: 0.5,
        );
        await tester.pumpAndSettle();
        await tester.tap(find.text('Allow input to this captured window'));
        await tester.pumpAndSettle();
        expect(bridge.calls.contains('desktopGrant'), isFalse);
        await tester.scrollUntilVisible(
          find.text('Enable selected-window access'),
          180,
        );
        await Scrollable.ensureVisible(
          tester.element(find.text('Enable selected-window access')),
          alignment: 0.5,
        );
        await tester.pumpAndSettle();
        await tester.tap(find.text('Enable selected-window access'));
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Window moved. Capture again'),
          findsOneWidget,
        );
        expect(local.draft, 'Keep my goal');
        await tester.scrollUntilVisible(
          find.byType(Image),
          -180,
          scrollable: find.byType(Scrollable).last,
        );
        expect(find.byType(Image), findsOneWidget);
        expect(bridge.calls.contains('start'), isFalse);
        expect(tester.takeException(), isNull);
      }
    },
  );
  testWidgets(
    'compact themes expose unsupported vision and never contact a model on open',
    (tester) async {
      tester.view.physicalSize = const Size(420, 480);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = DesktopBridge(), chat = ChatController(DesktopBridge());
      addTearDown(chat.dispose);
      for (final brightness in Brightness.values) {
        final local = ChatController(bridge);
        addTearDown(local.dispose);
        await tester.pumpWidget(
          MaterialApp(
            theme: ThemeData(brightness: brightness),
            home: DesktopSettingsInspector(chat: local),
          ),
        );
        await tester.pumpAndSettle();
        expect(
          find.textContaining('Side chats have no tool access.'),
          findsOneWidget,
        );
        expect(find.text('Analyze this screenshot'), findsOneWidget);
        expect(
          tester
              .widget<FilledButton>(
                find.widgetWithText(FilledButton, 'Analyze this screenshot'),
              )
              .onPressed,
          isNull,
        );
        expect(tester.takeException(), isNull);
        expect(bridge.calls.every((c) => c == 'desktopState'), isTrue);
      }
    },
  );
  testWidgets(
    'closed window recovery survives refresh and preserves draft with no silent retry',
    (tester) async {
      final bridge = DesktopBridge(), chat = ChatController(DesktopBridge());
      addTearDown(chat.dispose);
      final local = ChatController(bridge)
        ..session = 'one'
        ..workspaceRoot = 'synthetic'
        ..draft = 'Keep this draft';
      addTearDown(local.dispose);
      await tester.pumpWidget(
        MaterialApp(home: DesktopSettingsInspector(chat: local)),
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text('Refresh windows'));
      await tester.tap(find.text('Refresh windows'));
      await tester.pumpAndSettle();
      expect(
        find.text('Window closed. Refresh and capture again.'),
        findsOneWidget,
      );
      expect(local.draft, 'Keep this draft');
      expect(bridge.calls.where((c) => c == 'desktopObserve').length, 1);
      expect(bridge.calls.contains('start'), isFalse);
      await tester.ensureVisible(find.text('Refresh windows'));
      await tester.tap(find.text('Refresh windows'));
      await tester.pumpAndSettle();
      expect(bridge.calls.where((c) => c == 'desktopObserve').length, 2);
      expect(
        find.text('Window closed. Refresh and capture again.'),
        findsOneWidget,
      );
      expect(tester.takeException(), isNull);
    },
  );
}
