import 'dart:io';

import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/floating_notice.dart';

import 'background_host_test.dart' as availability show TestWindow;

import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/attention_host.dart';
import 'package:dolores_flutter/attention_panel.dart';
import 'package:dolores_flutter/notification_toast.dart';
import 'package:dolores_flutter/close_behavior.dart';
import 'package:dolores_flutter/general_notifications.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';

import 'app_host_test.dart' show HostBridge;
import 'attention_test.dart' show NoticeWindow;
import 'support/render_fonts.dart';

class NotificationBridge extends HostBridge {
  final notices = <Map<String, dynamic>>[];
  Map<String, dynamic> closePreference = {'revision': 0, 'behavior': 'ask'};
  bool fail = false;
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'] == 'backgroundPolicy') {
      return {'revision': 1, 'enabled': false};
    }
    if (c['command'] == 'closePreferences') return {...closePreference};
    if (c['command'] == 'saveClosePreferences') {
      if (fail) throw StateError('storage unavailable');
      closePreference = {
        ...c['preferences'] as Map<String, dynamic>,
        'revision': closePreference['revision'] + 1,
      };
      return {...closePreference};
    }
    if (c['command'] != 'attention') return super.call(c);
    final r = c['request'] as Map;
    switch (r['action']) {
      case 'post':
        if (fail) throw StateError('storage unavailable');
        final n = <String, dynamic>{
          'id': 'notice-${notices.length}',
          'title': r['title'],
          'details': r['details'],
          'severity': r['severity'],
          'kind': 'app',
          'createdAt': DateTime.now().millisecondsSinceEpoch,
          'seen': false,
          'delivery': 'suppressed',
        };
        notices.insert(0, n);
        return {...n};
      case 'poll':
        return {
          'preferences': {'enabled': false},
          'items': notices.map((n) => {...n}).toList(),
          'unread': notices.where((n) => n['seen'] != true).length,
          'pendingDelivery': [],
        };
      case 'seen':
        final n = notices.firstWhere((n) => n['id'] == r['id']);
        n['seen'] = true;
        return {...n};
      case 'seenAll':
        for (final n in notices) {
          n['seen'] = true;
        }
        return null;
      case 'clearRead':
        notices.removeWhere((n) => n['seen'] == true);
        return null;
      case 'savePreferences':
        return r['preferences'];
    }
    throw StateError('Unexpected notification action');
  }
}

Future<void> captureToast(WidgetTester t, GlobalKey key, String name) async {
  final directory = Platform.environment['DOLORES_RENDER_DIR'];
  if (directory == null) return;
  await t.runAsync(() async {
    final image =
        await (key.currentContext!.findRenderObject() as RenderRepaintBoundary)
            .toImage();
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    image.dispose();
    final file = File('$directory/$name.png');
    await file.parent.create(recursive: true);
    await file.writeAsBytes(data!.buffer.asUint8List());
  });
}

void main() {
  setUpAll(loadReviewFonts);
  testWidgets(
    'toast expands width then height, keeps content fixed, and exposes bounded details',
    (t) async {
      t.view.physicalSize = const Size(900, 650);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final bridge = NotificationBridge();
      final host = AttentionHost(
        bridge,
        window: NoticeWindow(),
        onOpen: (_) async => true,
      );
      final key = GlobalKey();
      await t.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: RepaintBoundary(
            key: key,
            child: NotificationOverlay(
              host: host,
              child: const Scaffold(
                body: Center(child: TextField(key: Key('stationary-field'))),
              ),
            ),
          ),
        ),
      );
      final before = t.getTopLeft(find.byKey(const Key('stationary-field')));
      await host.publish(
        'model',
        'Model settings saved',
        List.filled(80, 'Saved configuration details.').join(' '),
      );
      await t.pump();
      expect(
        t.getSize(find.byKey(const Key('notification-toast-surface'))),
        const Size(8, 8),
      );
      await t.pump(const Duration(milliseconds: 80));
      var size = t.getSize(find.byKey(const Key('notification-toast-surface')));
      expect(size.width, greaterThan(8));
      expect(size.height, 8);
      await captureToast(t, key, 'toast-horizontal');
      await t.pump(const Duration(milliseconds: 120));
      size = t.getSize(find.byKey(const Key('notification-toast-surface')));
      expect(size.width, 320);
      expect(size.height, greaterThan(8));
      await t.pumpAndSettle();
      expect(t.getTopLeft(find.byKey(const Key('stationary-field'))), before);
      expect(
        t.getSize(find.byKey(const Key('notification-toast-surface'))),
        const Size(320, 56),
      );
      await captureToast(t, key, 'toast-compact-dark');
      await t.tap(find.byKey(const Key('notification-toast-open')));
      await t.pump();
      await t.pump(const Duration(milliseconds: 80));
      size = t.getSize(find.byKey(const Key('notification-toast-surface')));
      expect(size.width, greaterThan(320));
      expect(size.height, 56);
      await t.pumpAndSettle();
      expect(host.detailsOpen, true);
      expect(host.unread, 0);
      expect(
        t.getSize(find.byKey(const Key('notification-toast-surface'))),
        const Size(420, 280),
      );
      await captureToast(t, key, 'toast-details-dark');
      await t.pump(const Duration(seconds: 7));
      expect(host.toast, isNotNull);
      await t.tap(find.byTooltip('Dismiss notification'));
      await t.pumpAndSettle();
      expect(host.toast, isNull);
      expect(bridge.notices, hasLength(1));
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'expiry and upward swipe retain history, queue is bounded and reduced motion is immediate',
    (t) async {
      final bridge = NotificationBridge();
      final host = AttentionHost(
        bridge,
        window: NoticeWindow(),
        onOpen: (_) async => true,
      );
      final key = GlobalKey();
      await t.pumpWidget(
        MaterialApp(
          theme: doloresTheme(false),
          builder: (context, child) => MediaQuery(
            data: MediaQuery.of(context).copyWith(disableAnimations: true),
            child: child!,
          ),
          home: RepaintBoundary(
            key: key,
            child: NotificationOverlay(
              host: host,
              child: const Scaffold(body: Text('Saved settings remain here.')),
            ),
          ),
        ),
      );
      await host.publish('save', 'Settings saved', 'Your changes were saved.');
      await t.pumpAndSettle();
      expect(
        t.getSize(find.byKey(const Key('notification-toast-surface'))),
        const Size(320, 56),
      );
      await captureToast(t, key, 'toast-compact-light');
      await t.pump(const Duration(seconds: 6));
      await t.pump();
      expect(host.toast, isNull);
      expect(host.unread, 1);
      await host.publish('save', 'Another save', 'Changes were saved.');
      await t.pumpAndSettle();
      await t.drag(
        find.byKey(const Key('notification-toast-surface')),
        const Offset(0, -70),
      );
      await t.pumpAndSettle();
      expect(host.toast, isNull);
      expect(bridge.notices, hasLength(2));
      for (var i = 0; i < 10; i++) {
        await host.publish('save$i', 'Saved $i', 'Retained details.');
      }
      expect(host.queue.length, 4);
      expect(bridge.notices, hasLength(12));
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  testWidgets(
    'inbox details mark read, clear read preserves unread, and preferences live in General',
    (t) async {
      final bridge = NotificationBridge();
      final host = AttentionHost(
        bridge,
        window: NoticeWindow(),
        onOpen: (_) async => true,
      );
      final chat = ChatController(bridge)
        ..notifications = host
        ..loading = false;
      await host.publish(
        'save',
        'Saved model',
        'Connection choices were saved.',
      );
      host.dismissToast();
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: AttentionPanel(host: host)),
        ),
      );
      await t.pumpAndSettle();
      expect(find.text('Windows notifications'), findsNothing);
      await t.tap(find.byKey(const Key('notification-details-notice-0')));
      await t.pumpAndSettle();
      expect(host.detailsOpen, true);
      expect(host.unread, 0);
      await host.publish(
        'second',
        'Second notice',
        'Preserve this unread notice.',
      );
      await host.manage('clearRead');
      expect(bridge.notices, hasLength(1));
      expect(bridge.notices.single['seen'], false);
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: GeneralNotifications(chat: chat),
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      expect(find.text('Windows notifications'), findsOneWidget);
      expect(find.text('Ask every time'), findsOneWidget);
      await t.pumpWidget(const SizedBox());
      host.dispose();
      chat.dispose();
    },
  );
  testWidgets(
    'close chooser preserves Cancel and returns explicit remembered intent',
    (t) async {
      CloseChoice? choice;
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                onPressed: () async {
                  choice = await chooseCloseBehavior(context);
                },
                child: const Text('Close'),
              ),
            ),
          ),
        ),
      );
      await t.tap(find.text('Close'));
      await t.pumpAndSettle();
      await t.tap(find.text('Cancel'));
      await t.pumpAndSettle();
      expect(choice, isNull);
      await t.tap(find.text('Close'));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('remember-close-choice')));
      await t.tap(find.byKey(const Key('close-choice-tray')));
      await t.pumpAndSettle();
      expect(choice!.behavior, 'tray');
      expect(choice!.remember, true);
    },
  );
  testWidgets(
    'remembered tray leaves a running owner intact and General resets saved choices',
    (t) async {
      final bridge = NotificationBridge();
      final chat = ChatController(bridge)
        ..loading = false
        ..busy = true
        ..draft = 'keep active work';
      final window = availability.TestWindow();
      final host = AppHost(
        chat,
        availabilityWindow: window,
        attentionWindow: NoticeWindow(),
      );
      await t.pumpWidget(
        MaterialApp(
          home: Builder(
            builder: (context) => Scaffold(
              body: TextButton(
                onPressed: () => closeDolores(context, host),
                child: const Text('Close'),
              ),
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      await t.tap(find.text('Close'));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('remember-close-choice')));
      await t.tap(find.byKey(const Key('close-choice-tray')));
      await t.pumpAndSettle();
      expect(bridge.closePreference['behavior'], 'tray');
      expect(host.background.hidden, true);
      expect(chat.busy, true);
      expect(chat.draft, 'keep active work');
      await window.restore();
      await t.pumpAndSettle();
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SingleChildScrollView(
              child: GeneralNotifications(chat: chat),
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      await t.tap(find.text('Minimize to system tray'));
      await t.pumpAndSettle();
      await t.tap(find.text('Ask every time').last);
      await t.pumpAndSettle();
      expect(bridge.closePreference['behavior'], 'ask');
      bridge.fail = true;
      await t.tap(find.text('Ask every time'));
      await t.pumpAndSettle();
      await t.tap(find.text('Close the app').last);
      await t.pumpAndSettle();
      expect(bridge.closePreference['behavior'], 'ask');
      expect(
        find.textContaining('Could not save the close preference'),
        findsOneWidget,
      );
      await t.pumpWidget(const SizedBox());
      chat.busy = false;
      host.dispose();
    },
  );
  testWidgets(
    'existing floating notices use the same toast and preserve explicit Retry',
    (t) async {
      final bridge = NotificationBridge();
      final host = AttentionHost(
        bridge,
        window: NoticeWindow(),
        onOpen: (_) async => true,
      );
      var retries = 0;
      await t.pumpWidget(
        MaterialApp(
          home: NotificationOverlay(
            host: host,
            child: Scaffold(
              body: FloatingNotice(
                message: 'A public operation failed. Your draft remains.',
                onDismiss: () {},
                onRetry: () => retries++,
              ),
            ),
          ),
        ),
      );
      await t.pumpAndSettle();
      expect(find.text('Dolores update'), findsOneWidget);
      expect(bridge.notices, hasLength(1));
      await t.tap(find.byKey(const Key('notification-toast-open')));
      await t.pumpAndSettle();
      await t.tap(find.text('Retry'));
      await t.pump();
      expect(retries, 1);
      await t.pumpWidget(const SizedBox());
      expect(host.actions, isEmpty);
      host.dispose();
    },
  );
}
