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

void main() {
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
