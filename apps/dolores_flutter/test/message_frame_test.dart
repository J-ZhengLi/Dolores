import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/message_frame.dart';
import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'widget_test.dart' show FakeBridge;

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'messages align by role; hover does not move long compact text ($dark)',
      (tester) async {
        tester.view.physicalSize = const Size(390, 800);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final chat = ChatController(FakeBridge())..loading = false;
        chat.messages = [
          {
            'id': 1,
            'role': 'user',
            'content': 'A long path ${'x' * 100}\n**literal prompt**',
            'savedAt': DateTime(2026, 10, 5, 14, 7).millisecondsSinceEpoch,
          },
          {
            'id': 2,
            'role': 'assistant',
            'content': 'A short reply',
            'savedAt': DateTime(2026, 10, 5, 14, 8).millisecondsSinceEpoch,
          },
        ];
        await tester.pumpWidget(
          DoloresApp(
            chat: chat,
            themeMode: dark ? ThemeMode.dark : ThemeMode.light,
          ),
        );
        await tester.pumpAndSettle();
        final frames = find.byType(MessageFrame);
        final user = find.descendant(
          of: frames.first,
          matching: find.byKey(const Key('message-body')),
        );
        final assistant = find.descendant(
          of: frames.last,
          matching: find.byKey(const Key('message-body')),
        );
        final before = tester.getRect(user);
        expect(before.right, closeTo(tester.getRect(frames.first).right, .1));
        expect(before.left, greaterThan(tester.getRect(assistant).left));
        expect(
          find.descendant(of: frames, matching: find.text('You')),
          findsNothing,
        );
        expect(
          find.descendant(of: frames, matching: find.text('Dolores')),
          findsNothing,
        );
        expect(
          find.descendant(
            of: frames,
            matching: find.byIcon(Icons.person_outline),
          ),
          findsNothing,
        );
        expect(
          find.descendant(
            of: frames,
            matching: find.byIcon(Icons.all_inclusive_rounded),
          ),
          findsNothing,
        );
        expect(find.byTooltip('Copy message').hitTestable(), findsNothing);
        final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
        await mouse.addPointer(location: Offset.zero);
        await mouse.moveTo(tester.getCenter(user));
        await tester.pump();
        expect(find.byTooltip('Copy message').hitTestable(), findsOneWidget);
        expect(find.text('2026-10-05 14:07').hitTestable(), findsOneWidget);
        expect(tester.getRect(user), before);
        await mouse.moveTo(const Offset(1, 1));
        await tester.pump();
        expect(find.byTooltip('Copy message').hitTestable(), findsNothing);
        expect(tester.getRect(user), before);
        expect(tester.takeException(), isNull);
        await mouse.removePointer();
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }

  testWidgets('keyboard users can reach hidden copy; exact source is copied', (
    tester,
  ) async {
    String? copied;
    tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
      SystemChannels.platform,
      (call) async {
        if (call.method == 'Clipboard.setData') {
          copied = (call.arguments as Map)['text'] as String;
        }
        return null;
      },
    );
    addTearDown(
      () => tester.binding.defaultBinaryMessenger.setMockMethodCallHandler(
        SystemChannels.platform,
        null,
      ),
    );
    await tester.pumpWidget(
      const MaterialApp(
        home: Scaffold(
          body: MessageFrame(
            user: true,
            text: '**original**\n```rust\nmain();\n```',
            child: Text('Message'),
          ),
        ),
      ),
    );
    expect(find.byTooltip('Copy message').hitTestable(), findsNothing);
    await tester.sendKeyEvent(LogicalKeyboardKey.tab);
    await tester.pump();
    expect(find.byTooltip('Copy message').hitTestable(), findsOneWidget);
    await tester.sendKeyEvent(LogicalKeyboardKey.tab);
    await tester.pump();
    await tester.sendKeyEvent(LogicalKeyboardKey.enter);
    await tester.pump();
    expect(copied, '**original**\n```rust\nmain();\n```');
    expect(tester.takeException(), isNull);
  });

  testWidgets(
    'unknown or invalid timestamps stay absent; streaming has no copy',
    (tester) async {
      for (final time in [null, -1, 9007199254740991]) {
        await tester.pumpWidget(
          MaterialApp(
            home: Scaffold(
              body: MessageFrame(
                user: false,
                text: 'reply',
                savedAt: time,
                child: const Text('reply'),
              ),
            ),
          ),
        );
        final mouse = await tester.createGesture(kind: PointerDeviceKind.mouse);
        await mouse.addPointer(location: Offset.zero);
        await mouse.moveTo(tester.getCenter(find.text('reply')));
        await tester.pump();
        expect(
          find.byType(Tooltip),
          findsOneWidget,
        ); // Copy only, no invented date.
        expect(tester.takeException(), isNull);
        await mouse.removePointer();
      }
      await tester.pumpWidget(
        const MaterialApp(
          home: Scaffold(
            body: MessageFrame(
              user: false,
              text: 'partial',
              streaming: true,
              child: Text('partial'),
            ),
          ),
        ),
      );
      expect(find.byTooltip('Copy message'), findsNothing);
      expect(tester.takeException(), isNull);
    },
  );
}
