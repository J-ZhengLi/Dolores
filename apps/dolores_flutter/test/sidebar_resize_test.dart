import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/chat_sidebar.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

class LocalBridge implements ChatBridge {
  @override
  Future<dynamic> call(Map<String, dynamic> command) async => null;
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
}

void main() {
  Future<ChatController> mount(WidgetTester tester) async {
    tester.view.physicalSize = const Size(1000, 700);
    tester.view.devicePixelRatio = 1;
    addTearDown(tester.view.resetPhysicalSize);
    addTearDown(tester.view.resetDevicePixelRatio);
    final chat = ChatController(LocalBridge())..loading = false;
    chat.draft = 'Keep this unfinished message';
    await tester.pumpWidget(DoloresApp(chat: chat));
    await tester.pumpAndSettle();
    addTearDown(chat.dispose);
    return chat;
  }

  double width(WidgetTester tester) =>
      tester.getSize(find.byType(ChatSidebar)).width;

  testWidgets('Divider returns to normal after release or cancelled drag', (
    tester,
  ) async {
    await mount(tester);
    final handle = find.byKey(const Key('sidebar-resize'));
    Color color() => tester
        .widget<Container>(find.byKey(const Key('sidebar-divider')))
        .color!;
    final p = Palette(
      Theme.of(tester.element(handle)).brightness == Brightness.dark,
    );
    for (final cancel in [false, true]) {
      final gesture = await tester.startGesture(tester.getCenter(handle));
      await gesture.moveBy(const Offset(50, 0));
      await tester.pump();
      expect(color(), p.accent);
      if (cancel) {
        await gesture.cancel();
      } else {
        await gesture.up();
      }
      await tester.pumpAndSettle();
      expect(color(), p.border);
    }
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowLeft);
    await tester.pumpAndSettle();
    expect(color(), p.border);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Drag grows and shrinks the sidebar; reset keeps the draft', (
    tester,
  ) async {
    await mount(tester);
    expect(width(tester), UiTokens.sidebarWidth);
    expect(tester.getTopLeft(find.text('Dolores')).dy, lessThan(24));
    await tester.drag(
      find.byKey(const Key('sidebar-resize')),
      const Offset(70, 0),
    );
    await tester.pumpAndSettle();
    expect(width(tester), greaterThan(UiTokens.sidebarWidth));
    await tester.drag(
      find.byKey(const Key('sidebar-resize')),
      const Offset(-40, 0),
    );
    await tester.pumpAndSettle();
    expect(width(tester), lessThan(322));
    final handle = find.byKey(const Key('sidebar-resize'));
    await tester.tap(handle);
    await tester.pump(const Duration(milliseconds: 100));
    await tester.tap(handle);
    await tester.pumpAndSettle();
    expect(width(tester), UiTokens.sidebarWidth);
    expect(find.text('Keep this unfinished message'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets('Extreme drags clamp and keyboard resizing stays usable', (
    tester,
  ) async {
    await mount(tester);
    await tester.drag(
      find.byKey(const Key('sidebar-resize')),
      const Offset(2000, 0),
    );
    await tester.pumpAndSettle();
    expect(width(tester), UiTokens.sidebarMaxWidth);
    await tester.drag(
      find.byKey(const Key('sidebar-resize')),
      const Offset(-2000, 0),
    );
    await tester.pumpAndSettle();
    expect(width(tester), UiTokens.sidebarMinWidth);
    await tester.sendKeyEvent(LogicalKeyboardKey.arrowRight);
    await tester.pumpAndSettle();
    expect(width(tester), UiTokens.sidebarMinWidth + 16);
    await tester.sendKeyEvent(LogicalKeyboardKey.home);
    await tester.pumpAndSettle();
    expect(width(tester), UiTokens.sidebarWidth);
    expect(find.text('Keep this unfinished message'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'Window shrink protects chat width and restores the chosen width',
    (tester) async {
      await mount(tester);
      await tester.drag(
        find.byKey(const Key('sidebar-resize')),
        const Offset(500, 0),
      );
      await tester.pumpAndSettle();
      tester.view.physicalSize = const Size(760, 480);
      await tester.pumpAndSettle();
      expect(width(tester), 279);
      expect(tester.takeException(), isNull);
      tester.view.physicalSize = const Size(700, 480);
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('sidebar-resize')), findsNothing);
      await tester.tap(find.byTooltip('Conversations'));
      await tester.pumpAndSettle();
      expect(width(tester), UiTokens.sidebarWidth);
      // Dismiss through Escape rather than mutating any chat selection.
      await tester.sendKeyEvent(LogicalKeyboardKey.escape);
      await tester.pumpAndSettle();
      tester.view.physicalSize = const Size(1000, 700);
      await tester.pumpAndSettle();
      expect(width(tester), UiTokens.sidebarMaxWidth);
      expect(find.text('Keep this unfinished message'), findsOneWidget);
      expect(tester.takeException(), isNull);
    },
  );
}
