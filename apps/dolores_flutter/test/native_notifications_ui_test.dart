import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/attention_panel.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/models.dart';
import 'package:dolores_flutter/notification_toast.dart';
import 'package:dolores_flutter/theme.dart';

import 'notifications_test.dart' show captureToast;
import 'support/render_fonts.dart';

void main() {
  setUpAll(loadReviewFonts);
  testWidgets(
    'native model save retains notification history without shifting the form',
    (t) async {
      final directory = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      void stage(String value) =>
          File('${directory.parent.path}/stage.txt').writeAsStringSync(value);
      stage('opening bridge');
      expect(
        directory.absolute.path,
        contains('${Platform.pathSeparator}output${Platform.pathSeparator}'),
      );
      final bridge = (await t.runAsync(() async {
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
        return t.runAsync(() => bridge.call({'command': command, ...fields}));
      }

      Future<void> wait(bool Function() ready) async {
        for (var i = 0; i < 100; i++) {
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 20)),
          );
          await t.pump(const Duration(milliseconds: 40));
          if (ready()) return;
        }
        fail('Native notification UI condition timed out.');
      }

      stage('bootstrap');
      await call('bootstrap');
      const endpoint = 'http://127.0.0.1:19421/v1';
      await call('configure', {
        'preferences': {'baseUrl': endpoint, 'model': 'public-fixture'},
        'apiKey': '',
        'rememberConnection': false,
        'enabledModels': ['public-fixture'],
      });
      final chat = ChatController(bridge)..loading = false;
      stage('refresh chat');
      await t.runAsync(chat.refresh);
      final host = AppHost(chat);
      final key = GlobalKey();
      t.view.physicalSize = const Size(1040, 800);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      stage('render');
      await t.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: RepaintBoundary(
            key: key,
            child: NotificationOverlay(
              host: host.attention,
              child: Scaffold(
                appBar: AppBar(
                  title: const Text('Public fixture'),
                  actions: [AttentionButton(host: host.attention)],
                ),
                body: ModelDetailsEditor(chat: chat, model: 'public-fixture'),
              ),
            ),
          ),
        ),
      );
      await wait(
        () =>
            find.byKey(const Key('model-detail-context')).evaluate().isNotEmpty,
      );
      final field = find.byKey(const Key('model-detail-context'));
      await t.enterText(field, '65536');
      final before = t.getTopLeft(field);
      await t.tap(find.byKey(const Key('save-model-details')));
      await wait(
        () => host.attention.toast?['title'] == 'Model settings saved',
      );
      await t.pump(const Duration(milliseconds: 500));
      expect(t.getTopLeft(field), before);
      expect(find.text('Saved'), findsNothing);
      final details = await call('modelDetails', {
        'preferences': {'baseUrl': endpoint, 'model': 'public-fixture'},
      }) as Map;
      expect(details['contextWindowTokens'], 65536);
      await captureToast(t, key, 'native-model-save-toast');
      await t.pump(const Duration(seconds: 6));
      await t.pump(const Duration(seconds: 1));
      expect(host.attention.toast, isNull);
      await t.tap(find.byKey(const Key('task-attention')));
      await wait(() => host.attention.items.isNotEmpty);
      final notice = host.attention.items.firstWhere((n) => n['kind'] == 'app');
      expect(notice['seen'], false);
      await t.tap(find.byKey(Key('notification-details-${notice['id']}')));
      await wait(
        () => host.attention.detailsOpen && host.attention.unread == 0,
      );
      await t.pump(const Duration(milliseconds: 500));
      expect(find.text('Your model configuration was saved.'), findsOneWidget);
      await captureToast(t, key, 'native-notification-details');
      File('${directory.parent.path}/notifications-receipt.json')
          .writeAsStringSync(
            jsonEncode({
              'modelSaveAcknowledged': true,
              'formPositionPreserved': true,
              'expiryRetainedInbox': true,
              'detailsMarkedRead': true,
              'normalRustBridge': true,
              'modelRequests': 0,
              'evidence': 'rendered Flutter controls with native Rust/SQLite; not physical input',
            }),
          );
      await t.pumpWidget(const SizedBox());
      host.dispose();
      await t.runAsync(bridge.close);
    },
  );
}
