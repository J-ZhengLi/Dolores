import 'dart:ui' show ImageFilter;

import 'package:flutter/material.dart';

import 'theme.dart';

/// Portable shaped glass. Content stays crisp above a single bounded backdrop.
class LiquidGlass extends StatelessWidget {
  final Widget child;
  final double radius;
  final bool lens, blur;
  final Key? filterKey;
  const LiquidGlass({
    super.key,
    required this.child,
    this.radius = 16,
    this.lens = false,
    this.blur = true,
    this.filterKey,
  });

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final media = MediaQuery.of(context);
    final opaque =
        media.highContrast ||
        media.disableAnimations ||
        media.accessibleNavigation;
    final shape = BorderRadius.circular(radius);
    final body = p.dark
        ? (lens ? const Color(0xff53575d) : p.sidebar)
        : (lens ? Colors.white : const Color(0xfff2f2ef));
    return DecoratedBox(
      decoration: BoxDecoration(
        borderRadius: shape,
        boxShadow: opaque
            ? null
            : [
                BoxShadow(
                  color: (p.dark ? Colors.black : const Color(0xff777870))
                      .withValues(alpha: p.dark ? .18 : .10),
                  blurRadius: lens ? 9 : 20,
                  offset: Offset(0, lens ? 2 : 6),
                ),
              ],
      ),
      child: ClipRRect(
        borderRadius: shape,
        child: BackdropFilter(
          key: filterKey,
          enabled: blur && !opaque,
          filter: ImageFilter.blur(sigmaX: 16, sigmaY: 16),
          child: CustomPaint(
            foregroundPainter: _GlassRim(p.dark, opaque, radius, lens, p.text),
            child: DecoratedBox(
              decoration: BoxDecoration(
                color: opaque ? body : null,
                gradient: opaque
                    ? null
                    : LinearGradient(
                        begin: Alignment.topLeft,
                        end: Alignment.bottomRight,
                        stops: const [0, .45, 1],
                        colors: [
                          body.withValues(
                            alpha: lens ? (p.dark ? .72 : .90) : .78,
                          ),
                          body.withValues(
                            alpha: lens ? (p.dark ? .32 : .50) : .60,
                          ),
                          body.withValues(
                            alpha: lens ? (p.dark ? .55 : .78) : .72,
                          ),
                        ],
                      ),
              ),
              child: child,
            ),
          ),
        ),
      ),
    );
  }
}

class _GlassRim extends CustomPainter {
  final bool dark, opaque, lens;
  final double radius;
  final Color contrast;
  const _GlassRim(
    this.dark,
    this.opaque,
    this.radius,
    this.lens,
    this.contrast,
  );

  @override
  void paint(Canvas canvas, Size size) {
    final rect = (Offset.zero & size).deflate(.6);
    final rim = RRect.fromRectAndRadius(rect, Radius.circular(radius));
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.1;
    if (opaque) {
      paint.color = contrast.withValues(alpha: lens ? .75 : .45);
      canvas.drawRRect(rim, paint);
      return;
    }
    // Directional rim and an inset shade imply depth without blurring the label.
    paint.shader = LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      stops: const [0, .35, .65, 1],
      colors: [
        Colors.white.withValues(alpha: dark ? .48 : .96),
        Colors.white.withValues(alpha: dark ? .12 : .32),
        const Color(0xff92948c).withValues(alpha: dark ? .18 : .30),
        Colors.white.withValues(alpha: dark ? .24 : .80),
      ],
    ).createShader(rect);
    canvas.drawRRect(rim, paint);
    paint
      ..strokeWidth = .7
      ..shader = LinearGradient(
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
        colors: [
          Colors.white.withValues(alpha: dark ? .08 : .55),
          Colors.black.withValues(alpha: dark ? .14 : .035),
        ],
      ).createShader(rect);
    canvas.drawRRect(rim.deflate(1.2), paint);
  }

  @override
  bool shouldRepaint(_GlassRim old) =>
      dark != old.dark ||
      opaque != old.opaque ||
      radius != old.radius ||
      lens != old.lens ||
      contrast != old.contrast;
}
