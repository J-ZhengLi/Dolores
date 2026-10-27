import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

Future<void> revealDock(WidgetTester tester) async {
  await tester.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
  await tester.sendKeyDownEvent(LogicalKeyboardKey.altLeft);
  await tester.sendKeyEvent(LogicalKeyboardKey.keyD);
  await tester.sendKeyUpEvent(LogicalKeyboardKey.altLeft);
  await tester.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
  // A running chat animates continuously. Wait for the dock's own transition,
  // rather than requiring the entire workspace to become idle.
  await tester.pump();
  await tester.pump(const Duration(milliseconds: 600));
}

Future<void> tapDockItem(WidgetTester tester, String key) async {
  await revealDock(tester);
  await tester.tap(find.byKey(Key(key)));
}
