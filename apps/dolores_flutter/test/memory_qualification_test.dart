import 'dart:io';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/memory.dart';
import 'package:dolores_flutter/theme.dart';

import 'memory_test.dart' show MemoryBridge;

void main() {
  final output = Platform.environment['DOLORES_MEMORY_RENDER_DIRECTORY'];
  setUpAll(() async {
    if (output != null) {
      final font = File('C:/Windows/Fonts/segoeui.ttf');
      if (await font.exists()) {
        final bytes = ByteData.sublistView(await font.readAsBytes());
        for (final family in ['MemoryRender', 'Segoe UI', 'Roboto']) {
          await (FontLoader(family)..addFont(Future.value(bytes))).load();
        }
        final icons = File(
          '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
        );
        if (await icons.exists()) {
          await (FontLoader('MaterialIcons')..addFont(
                Future.value(ByteData.sublistView(await icons.readAsBytes())),
              ))
              .load();
        }
      }
    }
  });
  for (final dark in [false, true]) {
    for (final width in [390.0, 920.0]) {
      testWidgets(
        'memory sources and revision history remain inspectable ($dark, $width)',
        (tester) async {
          tester.view.physicalSize = Size(width, 800);
          tester.view.devicePixelRatio = 1;
          addTearDown(tester.view.resetPhysicalSize);
          addTearDown(tester.view.resetDevicePixelRatio);
          final bridge = MemoryBridge()..supportsAutomatic = true;
          bridge.items.add({
            'id': 'fact',
            'revision': 2,
            'title': 'Fact: project codename',
            'text': 'Actually, our project codename is Birch instead.',
            'scope': 'folder',
            'source': 'automatic',
            'enabled': true,
            'updatedAt': 1,
            'createdAt': 1,
            'originAvailable': true,
            'confidence': 'Source-linked; inspect the exact statement',
            'origin': {
              'session': 'source',
              'messageId': 3,
              'quote': 'Actually, our project codename is Birch instead.',
              'model': 'Configured model',
            },
            'previous': [
              {
                'revision': 1,
                'title': 'Fact: project codename',
                'text': 'Our project codename is Cedar.',
              },
            ],
          });
          final key = GlobalKey();
          await tester.pumpWidget(
            RepaintBoundary(
              key: key,
              child: MaterialApp(
                debugShowCheckedModeBanner: false,
                theme: doloresTheme(dark).copyWith(
                  textTheme: doloresTheme(dark).textTheme.apply(
                    fontFamily: output == null ? null : 'MemoryRender',
                  ),
                  textButtonTheme: output == null
                      ? null
                      : TextButtonThemeData(
                          style: doloresTheme(dark).textButtonTheme.style
                              ?.copyWith(
                                textStyle: const WidgetStatePropertyAll(
                                  TextStyle(
                                    fontFamily: 'MemoryRender',
                                    fontSize: 14,
                                  ),
                                ),
                              ),
                        ),
                ),
                home: Scaffold(
                  body: MemoryInspector(bridge: bridge, session: 'chat'),
                ),
              ),
            ),
          );
          await tester.pumpAndSettle();
          expect(find.byKey(const Key('automatic-memory')), findsOneWidget);
          final history = find.text('Previous versions');
          await tester.scrollUntilVisible(
            history,
            100,
            scrollable: find.byType(Scrollable).first,
          );
          await Scrollable.ensureVisible(
            tester.element(history),
            alignment: 0.5,
          );
          await tester.pumpAndSettle();
          await tester.tap(history);
          await tester.pumpAndSettle();
          expect(find.text('Our project codename is Cedar.'), findsOneWidget);
          expect(find.text('Historical; excluded from recall'), findsOneWidget);
          if (output != null) {
            final boundary =
                key.currentContext!.findRenderObject()!
                    as RenderRepaintBoundary;
            await tester.runAsync(() async {
              final image = await boundary.toImage(pixelRatio: 1);
              final bytes = await image.toByteData(
                format: ui.ImageByteFormat.png,
              );
              final directory = Directory(output);
              await directory.create(recursive: true);
              await File(
                '${directory.path}/memory-${dark ? 'dark' : 'light'}-${width.toInt()}.png',
              ).writeAsBytes(bytes!.buffer.asUint8List());
              image.dispose();
            });
          }
          await Scrollable.ensureVisible(
            tester.element(find.byKey(const ValueKey('delete-memory-fact'))),
            alignment: 0.5,
          );
          await tester.pumpAndSettle();
          await tester.tap(find.byKey(const ValueKey('delete-memory-fact')));
          await tester.pumpAndSettle();
          expect(bridge.items, isEmpty);
          expect(
            find.textContaining('your conversations are unchanged'),
            findsOneWidget,
          );
          expect(tester.takeException(), isNull);
        },
      );
    }
  }
}
