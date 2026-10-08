import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/settings.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'companionship_test.dart' show CompanionBridge;
import 'settings_test.dart' show size;

void main() {
  for (final compact in [false, true]) {
    testWidgets(
      'Frequency search finds its control and retains a draft ($compact)',
      (tester) async {
        size(tester, compact ? const Size(420, 640) : const Size(1120, 780));
        final bridge = CompanionBridge();
        final chat = ChatController(bridge)..loading = false;
        await tester.pumpWidget(MaterialApp(home: SettingsWindow(chat: chat)));
        await tester.pumpAndSettle();
        final search = find.byKey(const Key('settings-search'));
        final result = find.byKey(
          const Key('setting-result-companionship-connection'),
        );
        for (final query in ['frequency', 'FREQUENCY', 'quiet', 'chatty']) {
          await tester.enterText(search, query);
          await tester.pumpAndSettle();
          expect(result, findsOneWidget, reason: 'Search: $query');
          expect(find.text('No matching settings'), findsNothing);
        }
        await tester.tap(result);
        await tester.pumpAndSettle();
        final frequency = find.byKey(const Key('companion-frequency'));
        await tester.ensureVisible(frequency);
        await tester.pumpAndSettle();
        await tester.drag(frequency, const Offset(1200, 0));
        await tester.pumpAndSettle();
        expect(tester.widget<Slider>(frequency).value, 100);

        await tester.enterText(search, 'no-such-setting');
        await tester.pumpAndSettle();
        expect(find.text('No matching settings'), findsOneWidget);
        await tester.enterText(search, 'Theme');
        await tester.pumpAndSettle();
        await tester.tap(
          find.byKey(const Key('setting-result-appearance-connection')),
        );
        await tester.pumpAndSettle();
        await tester.enterText(search, 'frequency');
        await tester.pumpAndSettle();
        await tester.tap(result);
        await tester.pumpAndSettle();
        expect(tester.widget<Slider>(frequency).value, 100);
        expect(
          bridge.commands.where((c) => c['command'] == 'companionPolicy'),
          isEmpty,
        );
        expect(tester.takeException(), isNull);
        await tester.pumpWidget(const SizedBox());
        chat.dispose();
      },
    );
  }
}
