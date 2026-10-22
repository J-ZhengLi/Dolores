import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/glass_navigation.dart';
import 'package:dolores_flutter/theme.dart';

class LayoutProbe extends SingleChildRenderObjectWidget {
  final VoidCallback onLayout;
  const LayoutProbe({super.key, required this.onLayout, required super.child});
  @override
  RenderObject createRenderObject(BuildContext context) => _Probe(onLayout);
}

class _Probe extends RenderProxyBox {
  final VoidCallback onLayout;
  _Probe(this.onLayout);
  @override
  void performLayout() {
    onLayout();
    super.performLayout();
  }
}

void main() {
  testWidgets('glass navigation hover does not paint a competing fill', (
    t,
  ) async {
    await t.pumpWidget(
      MaterialApp(
        theme: doloresTheme(false),
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 200,
              child: GlassNavigation(
                selected: 0,
                children: [
                  TextButton(onPressed: () {}, child: const Text('General')),
                  TextButton(onPressed: () {}, child: const Text('Access')),
                ],
              ),
            ),
          ),
        ),
      ),
    );
    final pointer = await t.createGesture(kind: PointerDeviceKind.mouse);
    await pointer.addPointer(location: Offset.zero);
    await pointer.moveTo(t.getCenter(find.text('Access')));
    await t.pumpAndSettle();
    final target = find.widgetWithText(TextButton, 'Access');
    final ink = t.widget<InkWell>(
      find.descendant(of: target, matching: find.byType(InkWell)),
    );
    final hover =
        ink.overlayColor?.resolve({WidgetState.hovered}) ?? ink.hoverColor;
    expect(
      hover?.a ?? 0,
      0,
      reason: 'The moving glass owns the fill, including hover/press',
    );
    await pointer.removePointer();
  });

  testWidgets('selection travel does not relayout stationary navigation', (
    t,
  ) async {
    var selected = 0;
    var layouts = 0;
    late StateSetter change;
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(
          body: Center(
            child: SizedBox(
              width: 200,
              child: LayoutProbe(
                onLayout: () => layouts++,
                child: StatefulBuilder(
                  builder: (context, update) {
                    change = update;
                    return GlassNavigation(
                      selected: selected,
                      children: [
                        for (var i = 0; i < 6; i++)
                          TextButton(onPressed: () {}, child: Text('Page $i')),
                      ],
                    );
                  },
                ),
              ),
            ),
          ),
        ),
      ),
    );
    await t.pumpAndSettle();
    change(() => selected = 5);
    await t.pump();
    layouts = 0;
    for (var i = 0; i < 20; i++) {
      await t.pump(const Duration(milliseconds: 16));
    }
    // This measures layout invalidation, not GPU frame time.
    debugPrint('Navigation layouts during 20 animation frames: $layouts');
    expect(layouts, 0);
  });
}
