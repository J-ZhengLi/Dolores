import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/glass_navigation.dart';
import 'package:dolores_flutter/liquid_glass.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/workspace_dock.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

void main() {
  testWidgets(
    'reversal and compact resize keep targets fixed; reduced motion interrupts travel',
    (t) async {
      var selected = 0;
      var reduced = false;
      var extent = 100.0;
      late StateSetter update;
      await t.pumpWidget(
        MaterialApp(
          theme: doloresTheme(false),
          home: StatefulBuilder(
            builder: (context, setState) {
              update = setState;
              return MediaQuery(
                data: MediaQuery.of(context)
                    .copyWith(disableAnimations: reduced),
                child: Scaffold(
                  body: Center(
                    child: GlassNavigation(
                      selected: selected,
                      axis: Axis.horizontal,
                      rowHeight: extent,
                      children: [
                        for (var i = 0; i < 3; i++)
                          TextButton(
                            onPressed: () => update(() => selected = i),
                            child: Text('Page $i'),
                          ),
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      );
      final target = t.getRect(find.text('Page 1'));
      final lens = find.byKey(const Key('glass-selection-position'));
      final origin = t.getCenter(lens);
      await t.tap(find.text('Page 2'));
      await t.pump();
      await t.pump(const Duration(milliseconds: 90));
      final moving = t.getCenter(lens);
      expect(moving.dx, greaterThan(origin.dx));
      expect(t.getRect(find.text('Page 1')), target);
      await t.tap(find.text('Page 0'));
      await t.pump();
      expect(t.getCenter(lens).dx, closeTo(moving.dx, .1));
      await t.pump(const Duration(milliseconds: 40));
      update(() {
        reduced = true;
        selected = 1;
      });
      await t.pumpAndSettle();
      expect(t.getCenter(lens).dx, closeTo(origin.dx + extent, .1));
      expect(t.widget<BackdropFilter>(lens).enabled, isFalse);
      expect(t.binding.transientCallbackCount, 0);
      update(() {
        reduced = false;
        selected = 2;
      });
      await t.pump();
      await t.pump(const Duration(milliseconds: 60));
      update(() => extent = 64);
      await t.pumpAndSettle();
      expect(
        t.getCenter(lens).dx,
        closeTo(t.getCenter(find.text('Page 2')).dx, 3),
      );
      expect(t.takeException(), isNull);
    },
  );

  testWidgets(
    'cancelled mouse press releases compression; contrast fallback stays selectable',
    (t) async {
      var highContrast = false;
      var selected = 0;
      late StateSetter update;
      await t.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: StatefulBuilder(
            builder: (context, setState) {
              update = setState;
              return MediaQuery(
                data: MediaQuery.of(context)
                    .copyWith(highContrast: highContrast),
                child: Scaffold(
                  body: SizedBox(
                    width: 220,
                    child: GlassNavigation(
                      selected: selected,
                      children: [
                        for (var i = 0; i < 2; i++)
                          TextButton(
                            onPressed: () => update(() => selected = i),
                            child: Text('Setting $i'),
                          ),
                      ],
                    ),
                  ),
                ),
              );
            },
          ),
        ),
      );
      final button = find.widgetWithText(TextButton, 'Setting 1');
      final scale = find.ancestor(
        of: button,
        matching: find.byType(AnimatedScale),
      );
      final press = await t.startGesture(
        t.getCenter(button),
        kind: PointerDeviceKind.mouse,
      );
      await t.pump(const Duration(milliseconds: 150));
      expect(t.widget<AnimatedScale>(scale).scale, lessThan(1));
      await press.cancel();
      await t.pumpAndSettle();
      expect(t.widget<AnimatedScale>(scale).scale, 1);
      expect(selected, 0);
      update(() => highContrast = true);
      await t.pumpAndSettle();
      await t.tap(button);
      await t.pumpAndSettle();
      expect(selected, 1);
      expect(
        t
            .widget<BackdropFilter>(
              find.byKey(const Key('glass-selection-position')),
            )
            .enabled,
        isFalse,
      );
      expect(t.takeException(), isNull);
    },
  );

  for (final dark in [false, true]) {
    testWidgets(
      'glass over busy content retains crisp controls and one dock blur ($dark)',
      (t) async {
        viewport(t, const Size(900, 480));
        debugDisableShadows = false;
        addTearDown(() => debugDisableShadows = true);
        await t.runAsync(loadReviewFonts);
        final key = GlobalKey();
        var selected = 0;
        late StateSetter update;
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: RepaintBoundary(
              key: key,
              child: StatefulBuilder(
                builder: (context, setState) {
                  update = setState;
                  final p = Palette(dark);
                  return Scaffold(
                    body: Stack(
                      children: [
                        Positioned.fill(
                          child: SingleChildScrollView(
                            child: Padding(
                              padding: const EdgeInsets.all(32),
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Text(
                                    'Project notes',
                                    style: Theme.of(context)
                                        .textTheme
                                        .headlineSmall,
                                  ),
                                  const SizedBox(height: 20),
                                  for (final name in [
                                    'Interface sketches',
                                    'Release checklist',
                                    'Recent changes',
                                    'Meeting notes',
                                    'Ideas to explore',
                                    'Draft release notes',
                                  ])
                                    Padding(
                                      padding: const EdgeInsets.only(
                                        bottom: 12,
                                      ),
                                      child: Container(
                                        height: 52,
                                        padding: const EdgeInsets.symmetric(
                                          horizontal: 16,
                                        ),
                                        decoration: BoxDecoration(
                                          color: p.surface,
                                          borderRadius: BorderRadius.circular(
                                            10,
                                          ),
                                          border: Border.all(color: p.border),
                                        ),
                                        child: Row(
                                          children: [
                                            Icon(
                                              Icons.description_outlined,
                                              color: p.muted,
                                            ),
                                            const SizedBox(width: 12),
                                            Text(name),
                                          ],
                                        ),
                                      ),
                                    ),
                                ],
                              ),
                            ),
                          ),
                        ),
                        Positioned(
                          left: 0,
                          right: 0,
                          bottom: 0,
                          child: WorkspaceDock(
                            selected: selected,
                            items: [
                              for (final (i, label, icon) in [
                                (0, 'Home', Icons.home_outlined),
                                (1, 'Files', Icons.folder_outlined),
                                (
                                  2,
                                  'Source Control',
                                  Icons.account_tree_outlined,
                                ),
                                (3, 'Settings', Icons.settings_outlined),
                              ])
                                DockItem(
                                  key: Key('sample-$i'),
                                  label: label,
                                  icon: icon,
                                  onPressed: () => update(() => selected = i),
                                ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  );
                },
              ),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(
          t
              .widgetList<BackdropFilter>(find.byType(BackdropFilter))
              .where((filter) => filter.enabled),
          hasLength(1),
        );
        await capture(t, key, 'material-${dark ? 'dark' : 'light'}');
        await t.tap(find.byKey(const Key('sample-2')));
        await t.pump();
        await t.pump(const Duration(milliseconds: 100));
        await capture(t, key, 'travel-${dark ? 'dark' : 'light'}');
        await t.pumpAndSettle();
        expect(selected, 2);
        expect(find.byType(LiquidGlass), findsNWidgets(2));
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        debugDisableShadows = true;
      },
    );
  }
}
