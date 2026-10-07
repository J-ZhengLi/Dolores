import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/settings.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'memory_test.dart' show MemoryBridge;
import 'workspace_shell_test.dart' show ExperimentalBridge;

class EmptyMemoryBridge extends MemoryBridge {
  EmptyMemoryBridge() {
    supportsAutomatic = true;
  }
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final result = await super.call(command);
    if (command['command'] == 'memories') result.remove('automaticAttempt');
    return result;
  }
}

void main() {
  final output = Platform.environment['DOLORES_SETTINGS_RENDER_DIRECTORY'];
  setUpAll(() async {
    if (output == null) return;
    final bytes = ByteData.sublistView(
      await File('C:/Windows/Fonts/segoeui.ttf').readAsBytes(),
    );
    for (final family in ['SettingsRender', 'Segoe UI', 'Roboto']) {
      await (FontLoader(family)..addFont(Future.value(bytes))).load();
    }
    final icons = await File(
      '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
    ).readAsBytes();
    await (FontLoader(
      'MaterialIcons',
    )..addFont(Future.value(ByteData.sublistView(icons)))).load();
  });

  for (final dark in [false, true]) {
    for (final width in [390.0, 940.0]) {
      for (final experimental in [false, true]) {
        testWidgets(
          'Concise settings and hover-only help ($dark, $width, $experimental)',
          (tester) async {
            tester.view.physicalSize = Size(width, 760);
            tester.view.devicePixelRatio = 1;
            addTearDown(tester.view.resetPhysicalSize);
            addTearDown(tester.view.resetDevicePixelRatio);
            final bridge = experimental
                ? ExperimentalBridge()
                : EmptyMemoryBridge();
            final chat = ChatController(bridge)..loading = false;
            final boundaryKey = GlobalKey();
            var theme = doloresTheme(dark);
            if (output != null) {
              theme = theme.copyWith(
                textTheme: theme.textTheme.apply(fontFamily: 'SettingsRender'),
                textButtonTheme: TextButtonThemeData(
                  style: theme.textButtonTheme.style?.copyWith(
                    textStyle: const WidgetStatePropertyAll(
                      TextStyle(fontFamily: 'SettingsRender', fontSize: 14),
                    ),
                  ),
                ),
                filledButtonTheme: FilledButtonThemeData(
                  style: theme.filledButtonTheme.style?.copyWith(
                    textStyle: const WidgetStatePropertyAll(
                      TextStyle(fontFamily: 'SettingsRender', fontSize: 14),
                    ),
                  ),
                ),
                tooltipTheme: theme.tooltipTheme.copyWith(
                  textStyle: TextStyle(
                    fontFamily: 'SettingsRender',
                    fontSize: 12,
                    color: dark ? Colors.black : Colors.white,
                  ),
                ),
              );
            }
            await tester.pumpWidget(
              RepaintBoundary(
                key: boundaryKey,
                child: MaterialApp(
                  debugShowCheckedModeBanner: false,
                  theme: theme,
                  home: SettingsWindow(
                    chat: chat,
                    initial: experimental
                        ? SettingsCategory.experimental
                        : SettingsCategory.memory,
                  ),
                ),
              ),
            );
            await tester.pumpAndSettle();
            expect(find.text('Memory details'), findsNothing);
            expect(find.text('Learning details'), findsNothing);
            expect(find.textContaining('performance checks'), findsNothing);
            final toggle = find.byKey(
              Key(
                experimental
                    ? 'experimental-multiple-window'
                    : 'automatic-memory',
              ),
            );
            final help = find.byKey(
              Key(experimental ? 'multiple-window-help' : 'memory-help'),
            );
            await tester.scrollUntilVisible(
              help,
              100,
              scrollable: find.byType(Scrollable).last,
            );
            await tester.pumpAndSettle();
            final tooltip = tester.widget<Tooltip>(
              find.ancestor(of: help, matching: find.byType(Tooltip)),
            );
            final value = tester.widget<SwitchListTile>(toggle).value;
            expect(tooltip.triggerMode, TooltipTriggerMode.manual);
            expect(find.text(tooltip.message!), findsNothing);
            await tester.tap(help);
            await tester.longPress(help);
            await tester.pumpAndSettle();
            expect(find.text(tooltip.message!), findsNothing);
            expect(tester.widget<SwitchListTile>(toggle).value, value);
            Future<void> render(String suffix) async {
              if (output == null) return;
              final boundary =
                  boundaryKey.currentContext!.findRenderObject()
                      as RenderRepaintBoundary;
              await tester.runAsync(() async {
                final image = await boundary.toImage();
                final bytes = await image.toByteData(
                  format: ui.ImageByteFormat.png,
                );
                final directory = Directory(output);
                await directory.create(recursive: true);
                await File(
                  '${directory.path}/${experimental ? 'experimental' : 'memory'}-${dark ? 'dark' : 'light'}-${width.toInt()}$suffix.png',
                ).writeAsBytes(bytes!.buffer.asUint8List());
                image.dispose();
              });
            }

            await render('');
            final before = tester.getRect(toggle);
            final mouse = await tester.createGesture(
              kind: ui.PointerDeviceKind.mouse,
            );
            await mouse.addPointer(location: Offset.zero);
            await mouse.moveTo(tester.getCenter(help));
            await tester.pump(const Duration(milliseconds: 700));
            await tester.pumpAndSettle();
            expect(find.text(tooltip.message!), findsOneWidget);
            expect(tester.getRect(toggle), before);
            expect(tester.widget<SwitchListTile>(toggle).value, value);
            if (dark && width == 940) await render('-hint');
            await mouse.moveTo(Offset.zero);
            await tester.pump(const Duration(milliseconds: 300));
            await tester.pumpAndSettle();
            expect(find.text(tooltip.message!), findsNothing);
            await mouse.removePointer();
            expect(tester.takeException(), isNull);
            await tester.pumpWidget(const SizedBox());
            chat.dispose();
          },
        );
      }
    }
  }
}
