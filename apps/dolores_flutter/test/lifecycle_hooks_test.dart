import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/lifecycle_hooks.dart';
import 'package:dolores_flutter/theme.dart';

import 'support/render_fonts.dart';

class HookBridge implements ChatBridge {
  List<Map<String, dynamic>> saved = [];
  Map<String, dynamic>? review;
  int revision = 0;
  bool fail = false;
  final calls = <Map<String, dynamic>>[];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command);
    final request = command['request'] as Map;
    if (request['action'] == 'review') {
      review = {
        'token': 'review',
        'root': 'C:/public/project',
        'configuration': {
          'revision': revision,
          'hooks': [
            for (final hook in request['hooks'] as List)
              {
                'hook': hook,
                'executable': 'C:/Runtime/node.exe',
                'scriptSha256': 'a' * 64,
              },
          ],
        },
        'scripts': [
          {
            'name': 'check',
            'text': 'console.log(JSON.stringify({allow:true,message:"ok"}));',
            'omittedBytes': 0,
          },
        ],
      };
      return review;
    }
    if (request['action'] == 'enable') {
      if (fail) throw 'Save failed. Your prior hooks remain; retry Enable.';
      saved = (review!['configuration']['hooks'] as List)
          .cast<Map<String, dynamic>>();
      revision++;
    }
    if (request['action'] == 'disable') {
      saved = [];
      revision++;
    }
    return {
      'configuration': {'revision': revision, 'hooks': saved},
    };
  }
}

Future<void> reveal(WidgetTester t, String key) async {
  final target = find.byKey(Key(key));
  if (target.evaluate().isEmpty) {
    t.state<ScrollableState>(find.byType(Scrollable).first).position.jumpTo(0);
    await t.pump();
    await t.scrollUntilVisible(
      target,
      160,
      scrollable: find.byType(Scrollable).first,
    );
  }
  await t.ensureVisible(target);
  await t.pump();
}

Future<void> press(WidgetTester t, String key) async {
  await reveal(t, key);
  await t.tap(find.byKey(Key(key)));
  await t.pumpAndSettle();
}

Future<void> fill(WidgetTester t, String key, String text) async {
  await reveal(t, key);
  await t.enterText(find.byKey(Key(key)), text);
  await t.pump();
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'Reviewed hooks preserve failed-save draft and disable recovery ${dark ? 'dark' : 'light'}',
      (t) async {
        if (const String.fromEnvironment('AUDIT_RENDER_DIR').isNotEmpty) {
          await t.runAsync(loadReviewFonts);
        }
        t.view.physicalSize = const Size(390, 700);
        t.view.devicePixelRatio = 1;
        addTearDown(t.view.resetPhysicalSize);
        addTearDown(t.view.resetDevicePixelRatio);
        final b = HookBridge();
        final render = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: RepaintBoundary(
                key: render,
                child: LifecycleHooksInspector(bridge: b, session: 'work'),
              ),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(b.calls.length, 1);
        await fill(t, 'hook-name', 'check');
        await fill(t, 'hook-script', 'scripts/check.cjs');
        await press(t, 'hook-stage');
        await press(t, 'hooks-review');
        expect(b.saved, isEmpty);
        expect(find.byKey(const Key('hooks-enable')), findsOneWidget);
        await t.scrollUntilVisible(
          find.text('Review automatic execution'),
          150,
          scrollable: find.byType(Scrollable).first,
        );
        await t.pumpAndSettle();
        const directory = String.fromEnvironment('AUDIT_RENDER_DIR');
        if (directory.isNotEmpty) {
          await t.runAsync(() async {
            final boundary =
                render.currentContext!.findRenderObject()
                    as RenderRepaintBoundary;
            final image = await boundary.toImage(pixelRatio: 1);
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            await Directory(directory).create(recursive: true);
            await File('$directory/hooks-review-${dark ? 'dark' : 'light'}.png')
                .writeAsBytes(bytes!.buffer.asUint8List());
            image.dispose();
          });
        }
        b.fail = true;
        await press(t, 'hooks-enable');
        expect(b.saved, isEmpty);
        expect(find.textContaining('Save failed.'), findsOneWidget);
        expect(find.byKey(const Key('hooks-enable')), findsOneWidget);
        b.fail = false;
        await press(t, 'hooks-enable');
        expect(b.saved.length, 1);
        expect(
          find.text('Hooks enabled for the next foreground task.'),
          findsOneWidget,
        );
        await fill(t, 'hook-name', 'unsaved');
        await press(t, 'hooks-disable');
        expect(b.saved, isEmpty);
        expect(find.text('All hooks disabled.'), findsOneWidget);
        expect(t.takeException(), isNull);
      },
    );
  }
}
