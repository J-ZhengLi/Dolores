import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/glass_navigation.dart';
import 'package:dolores_flutter/main.dart';

import 'app_host_test.dart' show HostBridge;
import 'support/render_fonts.dart';

class DesignBridge extends HostBridge {
  bool failSave = false;
  Map<String, dynamic> preferences = {
    'revision': 0,
    'multipleWindow': true,
    'preventWindowsFromLocked': false,
    'reduceMotion': false,
  };
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'saveExperimentalPreferences') {
      if (failSave) throw StateError('Storage unavailable');
      preferences = {
        ...command['preferences'] as Map<String, dynamic>,
        'revision': preferences['revision'] + 1,
      };
    }
    if (command['command'] == 'experimentalPreferences' ||
        command['command'] == 'saveExperimentalPreferences') {
      return {'preferences': preferences};
    }
    return super.call(command);
  }
}

void viewport(WidgetTester t, Size size) {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.resetPhysicalSize);
  addTearDown(t.view.resetDevicePixelRatio);
}

Future<void> capture(WidgetTester t, GlobalKey key, String name) async {
  const directory = String.fromEnvironment('DESIGN_RENDER_DIR');
  if (directory.isEmpty) return;
  await t.runAsync(() async {
    final boundary =
        key.currentContext!.findRenderObject() as RenderRepaintBoundary;
    final image = await boundary.toImage();
    final data = await image.toByteData(format: ui.ImageByteFormat.png);
    await Directory(directory).create(recursive: true);
    await File('$directory/$name.png').writeAsBytes(data!.buffer.asUint8List());
    image.dispose();
  });
}

void main() {
  testWidgets(
    'glass retargets interrupted input and reduced motion settles immediately',
    (t) async {
      int selected = 0;
      bool reduce = false;
      late StateSetter update;
      await t.pumpWidget(
        MaterialApp(
          home: StatefulBuilder(
            builder: (context, setState) {
              update = setState;
              return MediaQuery(
                data: MediaQuery.of(context)
                    .copyWith(disableAnimations: reduce),
                child: Center(
                  child: SizedBox(
                    width: 220,
                    child: GlassNavigation(
                      selected: selected,
                      children: [
                        for (var i = 0; i < 3; i++)
                          TextButton(onPressed: () {}, child: Text('Page $i')),
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      );
      final glass = find.byType(BackdropFilter);
      final start = t.getTopLeft(glass).dy;
      update(() => selected = 2);
      await t.pump();
      await t.pump(const Duration(milliseconds: 100));
      final moving = t.getTopLeft(glass).dy;
      expect(moving, greaterThan(start));
      update(() => selected = 1);
      await t.pump();
      expect(t.getTopLeft(glass).dy, closeTo(moving, .1));
      await t.pumpAndSettle();
      expect(t.getTopLeft(glass).dy, closeTo(start + 48, .1));
      update(() {
        reduce = true;
        selected = 0;
      });
      await t.pump();
      await t.pump();
      expect(t.getTopLeft(glass).dy, closeTo(start, .1));
      expect(t.widget<BackdropFilter>(glass).enabled, isFalse);
      expect(t.binding.transientCallbackCount, 0);
    },
  );

  testWidgets(
    'failed motion save retains choice and retry updates every conversation',
    (t) async {
      viewport(t, const Size(1100, 800));
      final bridge = DesignBridge()..failSave = true;
      final chat = ChatController(bridge)..loading = false;
      final host = AppHost(chat);
      await t.pumpWidget(DoloresApp(chat: chat, host: host));
      await t.tap(find.byKey(const Key('rail-settings')));
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('gentle-motion')));
      await t.pumpAndSettle();
      expect(chat.reducedMotion.value, isFalse);
      expect(find.textContaining("Couldn't save motion"), findsOneWidget);
      bridge.failSave = false;
      await t.tap(find.byKey(const Key('gentle-motion')));
      await t.pumpAndSettle();
      expect(chat.reducedMotion.value, isTrue);
      expect(bridge.preferences['multipleWindow'], isTrue);
      await t.tap(find.byKey(const Key('close-settings')));
      await t.pumpAndSettle();
      await host.newConversation(kind: 'side');
      await t.pumpAndSettle();
      expect(host.visible.reducedMotion.value, isTrue);
      expect(
        MediaQuery.of(t.element(find.byType(GlassNavigation)))
            .disableAnimations,
        isTrue,
      );
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );

  for (final mode in [ThemeMode.light, ThemeMode.dark]) {
    testWidgets(
      'native widgets retain draft through menus and centered settings ($mode)',
      (t) async {
        viewport(t, const Size(1200, 900));
        debugDisableShadows = false;
        addTearDown(() => debugDisableShadows = true);
        await t.runAsync(loadReviewFonts);
        final chat = ChatController(DesignBridge())
          ..loading = false
          ..configured = true
          ..attachmentsAvailable = true
          ..model = 'Everyday'
          ..enabledModels = ['Everyday', 'Thoughtful']
          ..draft = 'Help me improve my notes app.'
          ..projects = [
            {'name': 'Notes app', 'root': 'C:/fixture/notes'},
            {'name': 'A little inspiration', 'root': 'C:/fixture/ideas'},
            {'name': 'Personal website', 'root': 'C:/fixture/site'},
          ];
        final host = AppHost(chat);
        final key = GlobalKey();
        await t.pumpWidget(
          DoloresApp(chat: chat, host: host, themeMode: mode, captureKey: key),
        );
        await t.pumpAndSettle();
        expect(t.takeException(), isNull);
        await capture(t, key, 'home-${mode.name}');
        await t.tap(find.byKey(const Key('chat-model-picker')));
        await t.pumpAndSettle();
        expect(find.byType(Dialog), findsNothing);
        await capture(t, key, 'model-${mode.name}');
        await t.sendKeyEvent(LogicalKeyboardKey.escape);
        await t.pumpAndSettle();
        expect(chat.draft, 'Help me improve my notes app.');
        await t.tap(find.byKey(const Key('rail-settings')));
        await t.pumpAndSettle();
        final rect = t.getRect(find.byKey(const Key('settings-window')));
        expect(rect.center.dx, closeTo(600, 1));
        await capture(t, key, 'settings-${mode.name}');
        await t.tap(find.byKey(const Key('settings-memory')));
        await t.pumpAndSettle();
        expect(chat.draft, 'Help me improve my notes app.');
        await t.tap(find.byKey(const Key('close-settings')));
        await t.pumpAndSettle();
        t.view.physicalSize = const Size(420, 640);
        host.togglePanel();
        await t.pumpAndSettle();
        expect(t.takeException(), isNull);
        await capture(t, key, 'compact-${mode.name}');
        await t.tap(find.byKey(const Key('chat-model-picker')));
        await t.pumpAndSettle();
        final option = t.getRect(find.text('Thoughtful').last);
        expect(option.left, greaterThanOrEqualTo(0));
        expect(option.right, lessThanOrEqualTo(420));
        expect(option.bottom, lessThanOrEqualTo(640));
        await t.sendKeyEvent(LogicalKeyboardKey.escape);
        await t.pumpAndSettle();
        expect(chat.draft, 'Help me improve my notes app.');
        await t.pumpWidget(const SizedBox());
        host.dispose();
        debugDisableShadows = true;
      },
    );
  }
}
