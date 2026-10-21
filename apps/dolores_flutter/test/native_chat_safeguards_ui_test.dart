import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/models.dart';
import 'package:dolores_flutter/session_summary.dart';
import 'package:dolores_flutter/settings_frame.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/render_fonts.dart';

void main() {
  setUpAll(() async {
    if (Platform.environment['DOLORES_AUDIT_LIBRARY'] == null) return;
    await loadReviewFonts();
  });
  testWidgets(
    'Native chat reentry, summary safeguard and model removal',
    (tester) async {
      final data = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        data.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final bridge = (await tester.runAsync(() async {
        final bridge = NativeBridge();
        await bridge.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        return bridge;
      }))!;
      Future<dynamic> call(
        String command, [
        Map<String, dynamic> fields = const {},
      ]) async {
        final result = (await tester.runAsync(() async {
          try {
            return {
              'value': await bridge.call({'command': command, ...fields}),
            };
          } catch (e) {
            return {'error': '$e'};
          }
        }))!;
        if (result.containsKey('error')) throw result['error']!;
        return result['value'];
      }

      Future<void> waitUntil(bool Function() ready) async {
        final end = DateTime.now().add(const Duration(seconds: 15));
        do {
          await tester.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 25)),
          );
          await tester.pump(const Duration(milliseconds: 100));
          if (ready()) return;
        } while (DateTime.now().isBefore(end));
        fail('Chat safeguard fixture did not finish');
      }

      final requests = <Map>[];
      final server = (await tester.runAsync(() async {
        final server = await HttpServer.bind(InternetAddress.loopbackIPv4, 0);
        server.listen((request) async {
          requests.add(
            jsonDecode(await utf8.decoder.bind(request).join()) as Map,
          );
          request.response.headers.contentType = ContentType(
            'text',
            'event-stream',
            charset: 'utf-8',
          );
          for (final delta in [
            {'content': 'Public fixture reply.'},
            <String, dynamic>{},
          ]) {
            request.response.write(
              'data: ${jsonEncode({
                'choices': [
                  {'index': 0, 'delta': delta, 'finish_reason': delta.isEmpty ? 'stop' : null},
                ],
              })}\n\n',
            );
          }
          request.response.write('data: [DONE]\n\n');
          await request.response.close();
        });
        return server;
      }))!;
      final chat = ChatController(bridge, ownsBridge: false)..loading = false;
      final lock = ValueNotifier(false);
      addTearDown(() async {
        chat.dispose();
        lock.dispose();
        await tester.runAsync(() async {
          await bridge.close();
          await server.close(force: true);
        });
      });
      await call('bootstrap');
      final policy = (await call('memories'))['automaticPolicy'];
      await call('setAutomaticMemory', {
        'enabled': false,
        'revision': policy['revision'],
      });
      final preferences = {
        'baseUrl': 'http://127.0.0.1:${server.port}/v1',
        'model': 'fixture',
      };
      await call('configure', {
        'preferences': preferences,
        'apiKey': '',
        'rememberConnection': true,
        'enabledModels': ['fixture', 'other'],
      });
      final session = (await call('createSession', {
        'kind': 'side',
      }))['session']['id'];
      await tester.runAsync(chat.refresh);
      await tester.runAsync(() => chat.select(session));
      tester.view.physicalSize = const Size(1120, 800);
      tester.view.devicePixelRatio = 1;
      addTearDown(tester.view.resetPhysicalSize);
      addTearDown(tester.view.resetDevicePixelRatio);
      final renderKey = GlobalKey();
      Future<void> capture(String name) async {
        await tester.pumpAndSettle();
        final boundary =
            renderKey.currentContext!.findRenderObject()
                as RenderRepaintBoundary;
        await tester.runAsync(() async {
          final picture = await boundary.toImage();
          try {
            final bytes = await picture.toByteData(
              format: ui.ImageByteFormat.png,
            );
            File('${data.parent.path}/$name.png')
                .writeAsBytesSync(bytes!.buffer.asUint8List());
          } finally {
            picture.dispose();
          }
        });
      }

      await tester.pumpWidget(
        RepaintBoundary(
          key: renderKey,
          child: DoloresApp(chat: chat),
        ),
      );
      await tester.pumpAndSettle();
      final field = find.byKey(const Key('composer-field-0'));
      await tester.enterText(field, 'Hello');
      await tester.pump();
      await tester.tap(find.byKey(const Key('send')));
      await waitUntil(
        () => !chat.busy && !chat.changing && chat.messages.length == 2,
      );
      await tester.pump(const Duration(seconds: 3));
      await tester.showKeyboard(field);
      const question = '你为什么叫 Dolores?';
      tester.testTextInput.updateEditingValue(
        const TextEditingValue(
          text: question,
          selection: TextSelection.collapsed(offset: question.length),
          composing: TextRange(start: 0, end: question.length),
        ),
      );
      await tester.pump();
      expect(
        tester.widget<IconButton>(find.byKey(const Key('send'))).onPressed,
        isNull,
      );
      final editor = tester.widget<TextField>(field);
      await chat.inspectLocalSettings(() async {
        await tester.pump();
        expect(tester.widget<TextField>(field).readOnly, isTrue);
        editor.focusNode!.unfocus();
        await tester.pump();
        editor.controller!.value = const TextEditingValue(
          text: question,
          selection: TextSelection.collapsed(offset: question.length),
        );
      });
      await tester.pump();
      expect(
        tester.widget<IconButton>(find.byKey(const Key('send'))).onPressed,
        isNotNull,
      );
      await capture('chat-send-recovered');
      await tester.tap(find.byKey(const Key('send')));
      await waitUntil(
        () => !chat.busy && !chat.changing && chat.messages.length == 4,
      );
      expect(requests.length, 2);
      expect(jsonEncode(requests.last), contains(question));
      chat.draft = 'Keep this unsent question';
      tester.view.physicalSize = const Size(420, 600);
      await tester.pumpWidget(
        RepaintBoundary(
          key: renderKey,
          child: MaterialApp(
            debugShowCheckedModeBanner: false,
            theme: doloresTheme(true),
            home: Scaffold(
              body: SessionSummaryInspector(bridge: bridge, session: session),
            ),
          ),
        ),
      );
      await waitUntil(
        () =>
            find.byKey(const Key('review-summary')).evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(find.byType(SwitchListTile), findsNothing);
      expect(
        (await call('reviewSummary', {'session': session}))['autoCompact'],
        isTrue,
      );
      await capture('summary-always-on');
      await tester.pumpWidget(
        RepaintBoundary(
          key: renderKey,
          child: MaterialApp(
            debugShowCheckedModeBanner: false,
            theme: doloresTheme(false),
            home: Scaffold(
              body: Builder(
                builder: (context) => SettingsEmbedding(
                  route: ModalRoute.of(context),
                  pending: lock,
                  child: ModelsSettings(chat: chat),
                ),
              ),
            ),
          ),
        ),
      );
      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('model-menu-fixture')));
      await tester.pumpAndSettle();
      await capture('models-remove-menu');
      await tester.tap(find.text('Remove model'));
      await waitUntil(() => !chat.changing && chat.model == 'other');
      expect((await call('bootstrap'))['preferences']['model'], 'other');
      await tester.tap(find.byKey(const Key('model-menu-other')));
      await tester.pumpAndSettle();
      await tester.tap(find.text('Remove model'));
      await waitUntil(() => !chat.changing && chat.enabledModels.isEmpty);
      await capture('models-empty');
      await call('recoverConnection');
      final state = await call('bootstrap');
      expect(state['enabledModels'], isEmpty);
      expect(state['configured'], isFalse);
      expect(
        (await call('messagesPage', {'session': session}))['items'].length,
        4,
      );
      expect(chat.draft, 'Keep this unsent question');
      File('${data.parent.path}/chat-safeguards-receipt.json')
          .writeAsStringSync(
            jsonEncode({
              'mode': 'Rendered Flutter and normal Rust bridge; synthetic IME updates and loopback provider',
              'providerRequests': requests.length,
              'sendRecovered': true,
              'compactionAlwaysOn': true,
              'removedAllModels': true,
              'historyAndDraftRetained': true,
            }),
          );
      expect(tester.takeException(), isNull);
      await tester.pumpWidget(const SizedBox());
    },
    skip:
        !Platform.isWindows ||
        Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
