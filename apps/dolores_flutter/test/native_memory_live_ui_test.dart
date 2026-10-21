import 'dart:convert';
import 'dart:io';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/memory.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

// Explicit opt-in only. The runner supplies a fresh profile and credentials in
// the child environment; no secret or endpoint is copied into the receipt.
void main() {
  testWidgets(
    'one Memory switch captures facts and preferences from real chats',
    (tester) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        directory.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final model = Platform.environment['DOLORES_AUDIT_LIVE_MODEL']!;
      final base = Platform.environment['DOLORES_AUDIT_LIVE_BASE']!;
      final bridge = (await tester.runAsync(() async {
        final b = NativeBridge();
        await b.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return b;
      }))!;
      Future<dynamic> call(
        String command, [
        Map<String, dynamic> fields = const {},
      ]) async {
        final value = (await tester.runAsync(() async {
          try {
            return {
              'value': await bridge.call({'command': command, ...fields}),
            };
          } catch (e) {
            return {'error': e.toString()};
          }
        }))!;
        if (value.containsKey('error')) throw value['error']!;
        return value['value'];
      }

      Future<void> wait(bool Function() ready) async {
        final end = DateTime.now().add(const Duration(seconds: 35));
        do {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 30)),
          );
          await tester.pump(const Duration(milliseconds: 100));
          if (ready()) return;
        } while (DateTime.now().isBefore(end));
        fail('Bounded live UI operation did not finish');
      }

      Future<void> reveal(Finder finder) async {
        await tester.scrollUntilVisible(
          finder,
          150,
          scrollable: find.byType(Scrollable).first,
        );
        await tester.ensureVisible(finder);
        await tester.pumpAndSettle();
      }

      await call('bootstrap');
      await call('configure', {
        'preferences': {'baseUrl': base, 'model': model},
        'apiKey': Platform.environment['DOLORES_AUDIT_LIVE_KEY'],
        'rememberConnection': false,
        'enabledModels': [model],
      });
      await call('setModelRequestSettings', {
        'preferences': {'baseUrl': base, 'model': model},
        'settings': {
          'maxOutputTokens': 512,
          'timeoutSeconds': 15,
          'reasoning': Platform.environment['DOLORES_AUDIT_LIVE_REASONING'],
        },
      });
      var policy = (await call('memories'))['automaticPolicy'];
      if (policy['enabled'] == true) {
        await call('setAutomaticMemory', {
          'enabled': false,
          'revision': policy['revision'],
        });
      }
      final source = (await call('createSession', {
        'kind': 'side',
      }))['session']['id'];
      tester.view.physicalSize = const Size(740, 840);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      await tester.pumpWidget(
        MaterialApp(
          home: MemoryInspector(bridge: bridge, session: source),
        ),
      );
      await wait(
        () => find.byKey(const Key('automatic-memory')).evaluate().isNotEmpty,
      );
      final toggle = find.byKey(const Key('automatic-memory'));
      expect(tester.widget<SwitchListTile>(toggle).value, isFalse);
      await tester.tap(toggle);
      await wait(() => tester.widget<SwitchListTile>(toggle).value);
      final chat = ChatController(bridge, ownsBridge: false)..loading = false;
      await tester.runAsync(chat.refresh);
      await tester.runAsync(() => chat.select(source));
      final turns = <Map<String, dynamic>>[];
      for (final prompt in [
        'Our public demo project codename is Willow-826. Acknowledge in one short sentence. No tools.',
        'I prefer concise explanations with a short example. Acknowledge in one short sentence. No tools.',
      ]) {
        await tester.pumpWidget(DoloresApp(chat: chat));
        await tester.pumpAndSettle();
        await tester.enterText(
          find.byKey(const Key('composer-field-0')),
          prompt,
        );
        await tester.pump();
        await tester.tap(find.byKey(const Key('send')));
        await wait(() => !chat.busy && !chat.changing);
        expect(chat.error, isNull);
        expect(chat.messages.last['role'], 'assistant');
        final message = chat.messages.last;
        Map? attempt;
        final end = DateTime.now().add(const Duration(seconds: 15));
        do {
          attempt =
              (await call('memories', {'session': source}))['automaticAttempt']
                  as Map?;
          if (attempt != null && attempt['status'] != 'updating') break;
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 40)),
          );
        } while (DateTime.now().isBefore(end));
        expect(attempt?['status'], 'completed');
        expect(attempt?['saved'], greaterThan(0));
        turns.add({
          'publicPrompt': prompt,
          'publicReply': message['content'],
          'usage': message['metadata']?['usage'],
          'automaticAttempt': attempt,
        });
      }
      final memories = await call('memories', {'session': source});
      expect(memories['items'], hasLength(2));
      expect(jsonEncode(memories['items']), contains('Willow-826'));
      expect(jsonEncode(memories['items']), contains('concise'));
      final preference = (memories['items'] as List).singleWhere(
        (item) => (item['text'] as String).contains('concise'),
      );
      expect(
        preference['text'],
        'I prefer concise explanations with a short example.',
      );
      await call('saveDraft', {
        'session': source,
        'text': 'Retain my next public draft',
      });
      await tester.pumpWidget(
        MaterialApp(
          home: MemoryInspector(bridge: bridge, session: source),
        ),
      );
      await wait(
        () => find
            .byKey(const Key('automatic-memory-attempt'))
            .evaluate()
            .isNotEmpty,
      );
      final activity = find.byKey(const Key('automatic-memory-attempt'));
      await reveal(activity);
      await tester.tap(find.text('Recent activity'));
      await tester.pumpAndSettle();
      expect(find.text('1 saved'), findsOneWidget);
      final evidence = <Map<String, dynamic>>[];
      for (final item in memories['items']) {
        final sourceButton = find.byKey(Key('source-memory-${item['id']}'));
        await reveal(sourceButton);
        await tester.tap(sourceButton);
        final body = find.byKey(Key('memory-evidence-${item['id']}'));
        await wait(() => body.evaluate().isNotEmpty);
        expect(
          tester.widget<SelectableText>(body).data,
          item['origin']['quote'],
        );
        evidence.add({
          'title': item['title'],
          'text': item['text'],
          'origin': item['origin'],
          'sourceExcerpt': tester.widget<SelectableText>(body).data,
        });
      }
      expect(
        await call('savedDraft', {'session': source}),
        'Retain my next public draft',
      );
      expect(
        (await call('messagesPage', {'session': source}))['items'],
        hasLength(4),
      );
      File('${directory.parent.path}/memory-live-receipt.json')
          .writeAsStringSync(
            const JsonEncoder.withIndent('  ').convert({
              'mode': 'Rendered Flutter controls, normal Rust bridge, explicitly configured real model; no native pointer claim',
              'model': model,
              'oneSwitchEnabled': true,
              'manuallySavedMemories': 0,
              'turns': turns,
              'memories': evidence,
              'activityVisible': true,
              'sourcesInspected': 2,
              'draftRetained': true,
            }),
          );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
      chat.dispose();
      await tester.runAsync(bridge.close);
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIVE_KEY'] == null,
  );
}
