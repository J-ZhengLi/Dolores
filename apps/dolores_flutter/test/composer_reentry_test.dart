import 'dart:async';

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/rich_composer.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'stop_recovery_test.dart';

void main() {
  testWidgets('An inactive block cannot clear another block IME composition', (
    tester,
  ) async {
    final chat = ChatController(InterruptedBridge(false))
      ..loading = false
      ..configured = true
      ..session = 'audit'
      ..draft = '# Title\n\nQuestion';
    await tester.pumpWidget(DoloresApp(chat: chat));
    await tester.pumpAndSettle();
    await tester.showKeyboard(find.byKey(const Key('composer-field-1')));
    tester.testTextInput.updateEditingValue(
      const TextEditingValue(
        text: 'Question',
        selection: TextSelection.collapsed(offset: 8),
        composing: TextRange(start: 0, end: 8),
      ),
    );
    await tester.pump();
    final heading = tester.widget<TextField>(
      find.byKey(const Key('composer-field-0')),
    );
    heading.controller!.selection = const TextSelection.collapsed(offset: 0);
    await tester.pump();
    expect(
      tester.widget<IconButton>(find.byKey(const Key('send'))).onPressed,
      isNull,
    );
    expect(chat.draft, '# Title\n\nQuestion');
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });

  for (final inspecting in [false, true]) {
    testWidgets(
      'IME commit after losing focus enables Send without changing text (inspecting=$inspecting)',
      (tester) async {
        final bridge = InterruptedBridge(false)..starts = 1;
        final chat = ChatController(bridge)
          ..loading = false
          ..configured = true
          ..session = 'audit';
        await tester.pumpWidget(DoloresApp(chat: chat));
        await tester.pumpAndSettle();
        final field = find.byKey(const Key('composer-field-0'));
        await tester.showKeyboard(field);
        const text = '你为什么叫 Dolores?';
        tester.testTextInput.updateEditingValue(
          const TextEditingValue(
            text: text,
            selection: TextSelection.collapsed(offset: text.length),
            composing: TextRange(start: 0, end: text.length),
          ),
        );
        await tester.pump();
        expect(
          tester.widget<IconButton>(find.byKey(const Key('send'))).onPressed,
          isNull,
        );
        final editor = tester.widget<TextField>(field);
        final inspected = Completer<void>();
        Future<void>? inspection;
        if (inspecting) {
          inspection = chat.inspectLocalSettings(() => inspected.future);
          await tester.pump();
          expect(tester.widget<TextField>(field).readOnly, isTrue);
        }
        editor.focusNode!.unfocus();
        await tester.pump();
        // Native IMEs can acknowledge the final composition after focus moves.
        editor.controller!.value = const TextEditingValue(
          text: text,
          selection: TextSelection.collapsed(offset: text.length),
        );
        if (inspecting) {
          inspected.complete();
          await inspection;
        }
        await tester.pump();
        expect(chat.draft, text);
        expect(
          tester.widget<IconButton>(find.byKey(const Key('send'))).onPressed,
          isNotNull,
        );
        await tester.tap(find.byKey(const Key('send')));
        await tester.pump(const Duration(milliseconds: 100));
        await tester.pumpAndSettle();
        expect(bridge.starts, 2);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }

  testWidgets('A completed reply allows another message after an idle delay', (
    tester,
  ) async {
    final bridge = InterruptedBridge(false)..starts = 1;
    final chat = ChatController(bridge)
      ..loading = false
      ..configured = true
      ..session = 'audit';
    await tester.pumpWidget(DoloresApp(chat: chat));
    await tester.pumpAndSettle();
    final field = find
        .descendant(
          of: find.byType(RichComposer),
          matching: find.byType(TextField),
        )
        .first;
    await tester.enterText(field, 'Hello');
    await tester.pump();
    expect(chat.draft, 'Hello');
    await tester.tap(find.byKey(const Key('send')));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pumpAndSettle();
    expect(bridge.starts, 2);
    await tester.pump(const Duration(seconds: 3));
    expect(chat.busy, isFalse);
    expect(chat.changing, isFalse);
    expect(tester.widget<TextField>(field).readOnly, isFalse);
    await tester.enterText(field, 'Another question');
    await tester.pump();
    expect(chat.draft, 'Another question');
    expect(
      tester.widget<IconButton>(find.byKey(const Key('send'))).onPressed,
      isNotNull,
    );
    await tester.tap(find.byKey(const Key('send')));
    await tester.pump(const Duration(milliseconds: 100));
    await tester.pumpAndSettle();
    expect(chat.error, isNull);
    expect(bridge.starts, 3);
    await tester.pumpWidget(const SizedBox());
    chat.dispose();
  });
}
