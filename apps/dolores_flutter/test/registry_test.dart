import 'package:dolores_flutter/capabilities.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'capabilities_test.dart' show CapabilityBridge;

void main() {
  testWidgets(
    'compact registry shows incompatible entry without disabling local inspection',
    (tester) async {
      tester.view.physicalSize = const Size(480, 600);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final bridge = CapabilityBridge();
      final chat = ChatController(bridge)..session = 'side';
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: CapabilitiesInspector(chat: chat),
        ),
      );
      await tester.pumpAndSettle();
      await tester.ensureVisible(find.text('Extension registry'));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Extension registry'));
      await tester.pumpAndSettle();
      expect(
        find.textContaining('fixture-extension · Unavailable'),
        findsOneWidget,
      );
      expect(find.textContaining('Unsupported host API'), findsOneWidget);
      await tester.tap(find.text('Refresh'));
      await tester.pumpAndSettle();
      expect(bridge.calls.where((c) => c == 'harnessInventory').length, 2);
      expect(tester.takeException(), isNull);
      expect(bridge.calls.contains('start'), isFalse);
    },
  );
}
