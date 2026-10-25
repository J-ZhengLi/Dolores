import 'dart:ui' show PointerDeviceKind;

import 'package:dolores_flutter/app_dropdown.dart';
import 'package:dolores_flutter/attention_host.dart';
import 'package:dolores_flutter/attention_panel.dart';
import 'package:dolores_flutter/help_tip.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'attention_test.dart' show AttentionBridge, NoticeWindow;
import 'frost_design_test.dart' show viewport, capture;
import 'support/render_fonts.dart';

void main() {
  setUpAll(loadReviewFonts);
  setUp(() {
    debugDisableShadows = false;
    addTearDown(() => debugDisableShadows = true);
  });
  for (final dark in [false, true]) {
    for (final width in [390.0, 1100.0]) {
      testWidgets(
        'anchored select fits edges, selects and restores focus ($dark, $width)',
        (t) async {
          viewport(t, Size(width, 700));
          final key = GlobalKey();
          var selected = 'Tools overview';
          await t.pumpWidget(
            MaterialApp(
              theme: doloresTheme(dark),
              builder: (_, child) => RepaintBoundary(key: key, child: child!),
              home: Scaffold(
                body: StatefulBuilder(
                  builder: (context, update) => Align(
                    alignment: Alignment.bottomRight,
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: SizedBox(
                        width: 350,
                        child: AppDropdown<String>(
                          key: const Key('select'),
                          initialValue: selected,
                          items: [
                            for (final label in [
                              'Tools overview',
                              'Web search',
                              'Browser',
                              'Computer use',
                              'Connections',
                              'Skills',
                            ])
                              DropdownMenuItem(
                                value: label,
                                child: Text(label),
                              ),
                          ],
                          onChanged: (value) => update(() => selected = value!),
                        ),
                      ),
                    ),
                  ),
                ),
              ),
            ),
          );
          await t.tap(find.byKey(const Key('select')));
          await t.pumpAndSettle();
          final choices = find.byType(MenuItemButton);
          expect(choices, findsNWidgets(6));
          for (final element in choices.evaluate()) {
            final rect = t.getRect(find.byWidget(element.widget));
            expect(rect.left, greaterThanOrEqualTo(0));
            expect(rect.right, lessThanOrEqualTo(width));
            expect(rect.bottom, lessThanOrEqualTo(700));
            expect(rect.top, greaterThanOrEqualTo(0));
          }
          await capture(t, key, 'select-${dark ? 'dark' : 'light'}-$width');
          await t.tap(find.widgetWithText(MenuItemButton, 'Connections'));
          await t.pumpAndSettle();
          expect(selected, 'Connections');
          expect(find.byType(MenuItemButton), findsNothing);
          await t.tap(find.byKey(const Key('select')));
          await t.pumpAndSettle();
          await t.sendKeyEvent(LogicalKeyboardKey.escape);
          await t.pumpAndSettle();
          expect(find.byType(MenuItemButton), findsNothing);
          expect(FocusManager.instance.primaryFocus?.hasFocus, true);
          await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
          await t.pumpAndSettle();
          expect(find.byType(MenuItemButton), findsNWidgets(6));
          await t.sendKeyEvent(LogicalKeyboardKey.arrowDown);
          await t.sendKeyEvent(LogicalKeyboardKey.enter);
          await t.pumpAndSettle();
          expect(find.byType(MenuItemButton), findsNothing);
          debugDisableShadows = true;
          expect(t.takeException(), isNull);
        },
      );

      testWidgets(
        'bell dropdown preserves updates and opens task ($dark, $width)',
        (t) async {
          viewport(t, Size(width, 700));
          final b = AttentionBridge();
          final opened = <Map<String, dynamic>>[];
          final h = AttentionHost(
            b,
            window: NoticeWindow(),
            onOpen: (target) async {
              opened.add(target);
              return true;
            },
          );
          final key = GlobalKey();
          await t.pumpWidget(
            MaterialApp(
              theme: doloresTheme(dark),
              builder: (_, child) => RepaintBoundary(key: key, child: child!),
              home: Scaffold(
                body: Align(
                  alignment: Alignment.topRight,
                  child: AttentionButton(host: h),
                ),
              ),
            ),
          );
          await t.tap(find.byKey(const Key('task-attention')));
          await t.pumpAndSettle();
          expect(find.byType(Dialog), findsNothing);
          expect(find.byType(AttentionPanel), findsOneWidget);
          final panel = t.getRect(find.byType(AttentionPanel));
          expect(panel.left, greaterThanOrEqualTo(0));
          expect(panel.right, lessThanOrEqualTo(width));
          expect(panel.bottom, lessThanOrEqualTo(700));
          expect(find.textContaining('Notifications show only'), findsNothing);
          final help = find.descendant(
            of: find.widgetWithText(HelpLabel, 'Windows notifications'),
            matching: find.byType(IconButton),
          );
          final mouse = await t.createGesture(kind: PointerDeviceKind.mouse);
          await mouse.addPointer(location: Offset.zero);
          await mouse.moveTo(t.getCenter(help));
          await t.pump(const Duration(milliseconds: 400));
          expect(
            find.textContaining('Notifications show only'),
            findsOneWidget,
          );
          await mouse.removePointer();
          Tooltip.dismissAllToolTips();
          await t.pumpAndSettle();
          await capture(t, key, 'bell-${dark ? 'dark' : 'light'}-$width');
          await t.tap(
            find.byKey(ValueKey('attention-open-${b.notices.single['id']}')),
          );
          await t.pumpAndSettle();
          expect(opened, hasLength(1));
          expect(find.byType(AttentionPanel), findsNothing);
          await t.tap(find.byKey(const Key('task-attention')));
          await t.pumpAndSettle();
          await t.sendKeyEvent(LogicalKeyboardKey.escape);
          await t.pumpAndSettle();
          expect(find.byType(AttentionPanel), findsNothing);
          debugDisableShadows = true;
          expect(t.takeException(), isNull);
          await t.pumpWidget(const SizedBox());
          h.dispose();
        },
      );
    }
  }
}
