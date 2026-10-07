import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/scheduled_host.dart';
import 'package:dolores_flutter/scheduled_page.dart';
import 'package:dolores_flutter/theme.dart';

import 'scheduled_host_test.dart' show ScheduleBridge;

class PageBridge extends ScheduleBridge {
  bool stale = false;
  @override
  Map get listing => {
    'items': [
      {
        'task': {
          'id': 'task',
          'revision': 1,
          'title': 'Daily report',
          'sourceSession': 'source',
          'paused': paused,
          'nextDue': 1791385200,
        },
        'receipt': {
          'schedule': 'Mon, Tue, Wed, Thu, Fri at 21:00 (Asia/Shanghai)',
          'model': 'fixture-model',
          'skill': 'daily-report',
          'project': 'C:/reports',
          'timezone': 'Asia/Shanghai',
        },
        'occurrences': [
          {
            'id': 'run',
            'due': 1791298800,
            'state': 'failed',
            'error': 'Model unavailable. Reconnect, then Run now.',
            'session': 'result',
          },
        ],
      },
    ],
  };
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    if (command['command'] == 'scheduledManage') {
      if (stale) throw StateError('Task changed. Refresh before trying again.');
      if (command['action'] == 'pause') paused = true;
      return listing;
    }
    return super.call(command);
  }
}

void main() {
  final output = Platform.environment['DOLORES_SCHEDULE_RENDER_DIRECTORY'];
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
      testWidgets('Management is concise and fits $width dark=$dark', (
        tester,
      ) async {
        tester.view.physicalSize = Size(width, 820);
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.resetPhysicalSize);
        addTearDown(tester.view.resetDevicePixelRatio);
        final bridge = PageBridge();
        final host = ScheduledHost(bridge, onClaim: (_) async {});
        await host.refresh();
        final key = GlobalKey();
        String? opened;
        final theme = doloresTheme(dark).copyWith(
          textButtonTheme: TextButtonThemeData(
            style: TextButton.styleFrom(
              textStyle: const TextStyle(fontFamily: 'Segoe UI', fontSize: 14),
            ),
          ),
        );
        await tester.pumpWidget(
          MaterialApp(
            theme: theme,
            home: Scaffold(
              body: RepaintBoundary(
                key: key,
                child: Material(
                  color: theme.scaffoldBackgroundColor,
                  child: Row(
                    children: [
                      if (width > 800)
                        SizedBox(
                          width: 252,
                          child: ScheduledPanel(
                            host: host,
                            selected: 'task',
                            onSelect: (_) {},
                          ),
                        ),
                      Expanded(
                        child: ScheduledPage(
                          host: host,
                          selected: 'task',
                          owners: [],
                          openResult: (s) async {
                            opened = s;
                          },
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        );
        await tester.pump();
        expect(tester.takeException(), isNull);
        expect(find.text('New task'), findsNothing);
        expect(find.text('Create'), findsNothing);
        expect(find.text('Runs while Dolores is open.'), findsOneWidget);
        await tester.tap(find.text('Open result'));
        expect(opened, 'result');
        if (output != null) {
          final boundary =
              key.currentContext!.findRenderObject() as RenderRepaintBoundary;
          await tester.runAsync(() async {
            final image = await boundary.toImage();
            final bytes = await image.toByteData(
              format: ui.ImageByteFormat.png,
            );
            image.dispose();
            final file = File(
              '$output/scheduled-${dark ? 'dark' : 'light'}-${width.toInt()}.png',
            );
            await file.parent.create(recursive: true);
            await file.writeAsBytes(bytes!.buffer.asUint8List());
          });
        }
        await tester.tap(find.byTooltip('Task actions'));
        await tester.pumpAndSettle();
        await tester.tap(find.text('Pause'));
        await tester.pumpAndSettle();
        expect(bridge.paused, true);
        host.dispose();
        await tester.pumpWidget(const SizedBox());
      });
    }
  }
  testWidgets('Stale action retains task and shows refresh recovery', (
    tester,
  ) async {
    final bridge = PageBridge()..stale = true;
    final host = ScheduledHost(bridge, onClaim: (_) async {});
    await host.refresh();
    await tester.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: ScheduledPage(
            host: host,
            selected: 'task',
            owners: [],
            openResult: (_) async {},
          ),
        ),
      ),
    );
    await tester.tap(find.byTooltip('Task actions'));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Pause'));
    await tester.pumpAndSettle();
    expect(host.items.length, 1);
    expect(
      find.textContaining('Task changed. Refresh before trying again.'),
      findsOneWidget,
    );
    expect(find.text('Refresh'), findsOneWidget);
    expect(bridge.paused, false);
    host.dispose();
    await tester.pumpWidget(const SizedBox());
  });
}
