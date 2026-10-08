import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/tool_activity.dart';
import 'package:dolores_flutter/theme.dart';

Map<String, dynamic> receipt({bool defaultTime = false}) => {
  'callId': 'creation',
  'name': 'schedule_task',
  'target': 'internal-plan-id',
  'status': 'completed',
  'content': jsonEncode({
    'task': 'saved-task',
    'schedule':
        'Mon, Tue, Wed, Thu, Fri at ${defaultTime ? '09:00' : '21:00'} (Asia/Shanghai)',
    'model': 'test-model',
    'destination': 'Scheduled → Results',
    'availability': 'Runs while Dolores is open.',
    'usedDefaultTime': defaultTime,
  }),
};

void main() {
  final output = Platform.environment['DOLORES_RECEIPT_RENDER_DIRECTORY'];
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
    for (final width in [390.0, 1100.0]) {
      testWidgets('saved schedule fits $width dark=$dark', (t) async {
        t.view.physicalSize = Size(width, 280);
        t.view.devicePixelRatio = 1;
        addTearDown(t.view.resetPhysicalSize);
        addTearDown(t.view.resetDevicePixelRatio);
        final key = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: RepaintBoundary(
                key: key,
                child: Padding(
                  padding: const EdgeInsets.all(16),
                  child: ToolRecords(records: [receipt(defaultTime: true)]),
                ),
              ),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(find.textContaining('09:00 (Asia/Shanghai)'), findsOneWidget);
        expect(find.text('Task scheduled'), findsOneWidget);
        expect(t.takeException(), isNull);
        if (output != null) {
          await t.runAsync(() async {
            final boundary =
                key.currentContext!.findRenderObject() as RenderRepaintBoundary;
            final picture = await boundary.toImage();
            final bytes = await picture.toByteData(
              format: ui.ImageByteFormat.png,
            );
            await Directory(output).create(recursive: true);
            await File(
              '$output/receipt-${width.toInt()}-${dark ? 'dark' : 'light'}.png',
            ).writeAsBytes(bytes!.buffer.asUint8List());
            picture.dispose();
          });
        }
      });
    }
  }
  testWidgets('saved schedule is readable without an assistant confirmation', (
    t,
  ) async {
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(body: ToolRecords(records: [receipt()])),
      ),
    );
    expect(find.text('Task scheduled'), findsOneWidget);
    expect(find.textContaining('21:00 (Asia/Shanghai)'), findsOneWidget);
    expect(find.text('internal-plan-id'), findsNothing);
    await t.tap(find.text('Task scheduled'));
    await t.pumpAndSettle();
    expect(find.textContaining('test-model'), findsOneWidget);
    expect(find.textContaining('Runs while Dolores is open.'), findsOneWidget);
  });

  testWidgets('default time is disclosed in the collapsed receipt', (t) async {
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ToolRecords(records: [receipt(defaultTime: true)]),
        ),
      ),
    );
    expect(find.textContaining('Default time'), findsOneWidget);
  });

  for (final dark in [false, true]) {
    for (final state in ['updated', 'paused', 'deleted']) {
      testWidgets('task $state names its saved outcome dark=$dark', (t) async {
        t.view.physicalSize = const Size(390, 560);
        t.view.devicePixelRatio = 1;
        addTearDown(t.view.resetPhysicalSize);
        addTearDown(t.view.resetDevicePixelRatio);
        final content = jsonDecode(receipt()['content'] as String) as Map;
        final record = {
          ...receipt(),
          'callId': state,
          'name': 'manage_scheduled_task',
          'content': jsonEncode({
            ...content,
            'paused': state == 'paused',
            'deleted': state == 'deleted',
            'nextRun': 1791464400,
          }),
        };
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(body: ToolRecords(records: [record])),
          ),
        );
        await t.pumpAndSettle();
        final title = 'Task $state';
        expect(find.text(title), findsOneWidget);
        expect(find.text('internal-plan-id'), findsNothing);
        expect(find.text('Task scheduled'), findsNothing);
        if (state != 'updated') {
          expect(
            find.textContaining(
              '${state == 'paused' ? 'Paused' : 'Deleted'} ·',
            ),
            findsOneWidget,
          );
        }
        await t.tap(find.text(title));
        await t.pumpAndSettle();
        expect(find.textContaining('Task ID: saved-task'), findsOneWidget);
        expect(
          find.textContaining('Next:'),
          state == 'updated' ? findsOneWidget : findsNothing,
        );
        expect(t.takeException(), isNull);
      });
    }
  }

  testWidgets(
    'failed task changes retain the actual error without claiming success',
    (t) async {
      for (final record in [
        {
          ...receipt(),
          'name': 'manage_scheduled_task',
          'status': 'blocked',
          'content': 'Task changed. Ask again.',
        },
        {...receipt(), 'name': 'manage_scheduled_task', 'content': '{broken'},
        {
          ...receipt(),
          'name': 'manage_scheduled_task',
          'content': jsonEncode({'schedule': 'daily'}),
        },
      ]) {
        await t.pumpWidget(
          MaterialApp(
            home: Scaffold(body: ToolRecords(records: [record])),
          ),
        );
        expect(find.text('Task updated'), findsNothing);
        expect(find.text('Task paused'), findsNothing);
        expect(find.text('Task deleted'), findsNothing);
        expect(find.text('internal-plan-id'), findsOneWidget);
        await t.tap(find.text('internal-plan-id'));
        await t.pumpAndSettle();
        expect(find.text(record['content'] as String), findsOneWidget);
        await t.pumpWidget(const SizedBox());
      }
    },
  );

  testWidgets('failed and malformed results never claim a saved task', (
    t,
  ) async {
    for (final record in [
      {
        ...receipt(),
        'status': 'failed',
        'content': 'Task could not be saved. Try again.',
      },
      {...receipt(), 'content': '{broken'},
      {
        ...receipt(),
        'content': jsonEncode({'schedule': 42}),
      },
    ]) {
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: ToolRecords(records: [record])),
        ),
      );
      expect(find.text('Task scheduled'), findsNothing);
    }
  });
}
