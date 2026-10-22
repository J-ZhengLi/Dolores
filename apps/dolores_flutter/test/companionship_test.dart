import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/companion_host.dart';
import 'package:dolores_flutter/companion_note.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/settings.dart';
import 'package:dolores_flutter/theme.dart';

import 'workspace_test.dart' show WorkspaceBridge;

class CompanionBridge extends WorkspaceBridge {
  bool fail = false, enabled = false, seen = false;
  int cap = 2;
  Map get data => {
    'available': true,
    'models': ['fixture-model'],
    'state': {
      'revision': 1,
      'policy': {
        'revision': 1,
        'enabled': enabled,
        'model': 'fixture-model',
        'zone': 'Asia/Shanghai',
        'start': 540,
        'end': 1260,
        'dailyCap': cap,
      },
      'activity': [
        {
          'id': 'note',
          'status': 'delivered',
          'kind': 'fact',
          'seen': seen,
          'session': 'from-dolores',
          'citation': 'https://science.nasa.gov/earth/facts/',
          'usage': null,
        },
      ],
    },
  };
  @override
  Future<dynamic> call(Map<String, dynamic> c) async {
    if (c['command'].toString().startsWith('companion')) {
      commands.add(c);
      if (fail) throw StateError('Save failed. Refresh and try again.');
      if (c['command'] == 'companionPolicy') {
        enabled = c['policy']['enabled'] as bool;
        cap = c['policy']['dailyCap'] as int;
      }
      if (c['command'] == 'companionFeedback') {
        seen = true;
        if (c['action'] == 'mute') enabled = false;
        if (c['action'] == 'fewer') cap = cap.clamp(0, 1);
      }
      return data;
    }
    return super.call(c);
  }
}

void main() {
  final output = Platform.environment['DOLORES_COMPANION_RENDER_DIRECTORY'];
  setUpAll(() async {
    if (output == null) return;
    final font = ByteData.sublistView(
      await File('C:/Windows/Fonts/segoeui.ttf').readAsBytes(),
    );
    for (final family in ['Segoe UI', 'Roboto']) {
      await (FontLoader(family)..addFont(Future.value(font))).load();
    }
    final icons = ByteData.sublistView(
      await File(
        '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
      ).readAsBytes(),
    );
    await (FontLoader('MaterialIcons')..addFont(Future.value(icons))).load();
  });
  for (final dark in [false, true]) {
    for (final width in [390.0, 1040.0]) {
      testWidgets('Concise settings preserve failed edits $width dark=$dark', (
        t,
      ) async {
        t.view.physicalSize = Size(width, 820);
        t.view.devicePixelRatio = 1;
        addTearDown(t.view.resetPhysicalSize);
        addTearDown(t.view.resetDevicePixelRatio);
        final b = CompanionBridge();
        final chat = ChatController(b)..loading = false;
        final key = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark).copyWith(
              textButtonTheme: TextButtonThemeData(
                style: TextButton.styleFrom(
                  textStyle: const TextStyle(
                    fontFamily: 'Segoe UI',
                    fontSize: 14,
                  ),
                ),
              ),
            ),
            home: RepaintBoundary(
              key: key,
              child: SettingsWindow(
                chat: chat,
                initial: SettingsCategory.companionship,
              ),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(t.takeException(), isNull);
        expect(find.text('Occasional notes from Dolores'), findsOneWidget);
        expect(find.textContaining('At least three hours'), findsNothing);
        final frequency = find.byKey(const Key('companion-frequency'));
        await t.scrollUntilVisible(frequency, 100,
          scrollable: find.descendant(of: find.byKey(const Key('companion-form')), matching: find.byType(Scrollable)).first);
        final slider = t.widget<Slider>(frequency);
        expect(slider.value, 2);
        expect(slider.min, 0);
        expect(slider.max, 100);
        final scroll = find
            .descendant(
              of: find.byKey(const Key('companion-form')),
              matching: find.byType(Scrollable),
            )
            .first;
        await t.scrollUntilVisible(
          find.text('Chatty · 100'),
          80,
          scrollable: scroll,
        );
        await t.pumpAndSettle();
        expect(find.text('Quiet · 0'), findsOneWidget);
        expect(find.text('Chatty · 100'), findsOneWidget);
        if (output != null) {
          final boundary =
              key.currentContext!.findRenderObject() as RenderRepaintBoundary;
          await t.runAsync(() async {
            final img = await boundary.toImage();
            final bytes = await img.toByteData(format: ui.ImageByteFormat.png);
            img.dispose();
            final f = File(
              '$output/companion-${dark ? 'dark' : 'light'}-${width.toInt()}.png',
            );
            await f.parent.create(recursive: true);
            await f.writeAsBytes(bytes!.buffer.asUint8List());
          });
        }
        await t.scrollUntilVisible(
          find.byKey(const Key('companion-enabled')),
          -80,
          scrollable: scroll,
        );
        await t.pumpAndSettle();
        await t.tap(find.byKey(const Key('companion-enabled')));
        await t.pump();
        await t.ensureVisible(frequency);
        await t.pumpAndSettle();
        await t.drag(frequency, const Offset(1200, 0));
        await t.pumpAndSettle();
        expect(t.widget<Slider>(frequency).value, 100);
        expect(find.text('100 / day'), findsOneWidget);
        b.fail = true;
        await t.scrollUntilVisible(
          find.byKey(const Key('save-companion')),
          250,
          scrollable: scroll,
        );
        await t.tap(find.byKey(const Key('save-companion')));
        await t.pumpAndSettle();
        expect(b.enabled, false);
        expect(b.cap, 2);
        await t.ensureVisible(frequency);
        await t.pumpAndSettle();
        expect(t.widget<Slider>(frequency).value, 100);
        await t.scrollUntilVisible(
          find.byKey(const Key('companion-enabled')),
          -250,
          scrollable: scroll,
        );
        expect(
          (t.widget(
            find.byKey(const Key('companion-enabled')),
          ) as SwitchListTile).value,
          true,
        );
        expect(find.textContaining('Save failed'), findsOneWidget);
        b.fail = false;
        await t.scrollUntilVisible(
          find.byKey(const Key('save-companion')),
          250,
          scrollable: scroll,
        );
        await t.tap(find.byKey(const Key('save-companion')));
        await t.pumpAndSettle();
        expect(b.enabled, true);
        expect(b.cap, 100);
        await t.ensureVisible(frequency);
        await t.pumpAndSettle();
        await t.drag(frequency, const Offset(-1200, 0));
        await t.pumpAndSettle();
        expect(t.widget<Slider>(frequency).value, 0);
        await t.ensureVisible(find.byKey(const Key('save-companion')));
        await t.pumpAndSettle();
        await t.tap(find.byKey(const Key('save-companion')));
        await t.pumpAndSettle();
        expect(b.cap, 0);
        await t.tap(find.text('Refresh'));
        await t.pumpAndSettle();
        await t.ensureVisible(frequency);
        expect(t.widget<Slider>(frequency).value, 0);
        await t.pumpWidget(const SizedBox());
        chat.dispose();
      });
    }
  }
  testWidgets('Unread card opens deliberately and Not now removes it', (
    t,
  ) async {
    final b = CompanionBridge();
    final h = CompanionHost(b, session: () => null, busy: () => false);
    await h.refresh();
    String? opened;
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: CompanionNote(
            host: h,
            open: (s) async {
              opened = s;
            },
          ),
        ),
      ),
    );
    await t.tap(find.text('Open'));
    expect(opened, 'from-dolores');
    expect(h.unread, isNotNull);
    await t.tap(find.byTooltip('Note options'));
    await t.pumpAndSettle();
    await t.tap(find.text('Not now'));
    await t.pumpAndSettle();
    expect(h.unread, isNull);
    expect(b.commands.last['action'], 'notNow');
    await t.pumpWidget(const SizedBox());
    h.dispose();
  });
  test('failed feedback retains unread note for retry', () async {
    final b = CompanionBridge();
    final h = CompanionHost(b, session: () => null, busy: () => false);
    await h.refresh();
    b.fail = true;
    await h.feedback('note', 'mute');
    expect(h.unread, isNotNull);
    expect(h.error, contains('failed'));
    b.fail = false;
    await h.feedback('note', 'fewer');
    expect(b.cap, 1);
    h.dispose();
  });
  for (final dark in [false, true]) {
    for (final action in [
      ('notNow', 'Not now'),
      ('dismiss', 'Dismiss'),
      ('fewer', 'Fewer messages'),
      ('mute', 'Turn off'),
    ]) {
      testWidgets(
        'compact note ${action.$2} retains failed action for explicit retry ($dark)',
        (t) async {
          t.view.physicalSize = const Size(390, 740);
          t.view.devicePixelRatio = 1;
          addTearDown(t.view.resetPhysicalSize);
          addTearDown(t.view.resetDevicePixelRatio);
          final b = CompanionBridge()
            ..enabled = true
            ..cap = 100;
          final h = CompanionHost(
            b,
            session: () => 'current-chat',
            busy: () => false,
          );
          await h.refresh();
          String? opened;
          await t.pumpWidget(
            MaterialApp(
              theme: doloresTheme(dark),
              home: Scaffold(
                body: CompanionNote(
                  host: h,
                  open: (s) async {
                    opened = s;
                  },
                ),
              ),
            ),
          );
          b.fail = true;
          await t.tap(find.byTooltip('Note options'));
          await t.pumpAndSettle();
          await t.tap(find.text(action.$2));
          await t.pumpAndSettle();
          expect(h.unread, isNotNull);
          expect(h.error, contains('Save failed'));
          expect(find.byIcon(Icons.error_outline), findsOneWidget);
          expect(b.enabled, isTrue);
          expect(b.cap, 100);
          final attempts = b.commands
              .where((c) => c['command'] == 'companionFeedback')
              .length;
          await t.pump(const Duration(seconds: 2));
          expect(
            b.commands.where((c) => c['command'] == 'companionFeedback').length,
            attempts,
          );
          b.fail = false;
          await t.tap(find.byTooltip('Note options'));
          await t.pumpAndSettle();
          await t.tap(find.text(action.$2));
          await t.pumpAndSettle();
          expect(b.commands.last['action'], action.$1);
          expect(h.unread, isNull);
          expect(h.error, isNull);
          expect(find.byType(CompanionNote), findsOneWidget);
          expect(find.text('A note from Dolores'), findsNothing);
          expect(
            opened,
            isNull,
            reason: 'Feedback must not open or send a chat',
          );
          expect(b.enabled, action.$1 != 'mute');
          expect(b.cap, action.$1 == 'fewer' ? 1 : 100);
          expect(
            b.commands.every(
              (c) => [
                'companionState',
                'companionFeedback',
              ].contains(c['command']),
            ),
            isTrue,
          );
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          h.dispose();
        },
      );
    }
  }
}
