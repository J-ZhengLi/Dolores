import 'package:flutter/material.dart';

/// Shared geometry for the native frame and the fallback workspace header.
class WindowTitle extends StatelessWidget {
  final VoidCallback? onTogglePanel;
  const WindowTitle({super.key, this.onTogglePanel});

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.only(left: 12),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.center,
      children: [
        const WindowIdentity(),
        if (onTogglePanel != null) ...[
          const SizedBox(width: 8),
          SizedBox(
            width: 32,
            height: 32,
            child: IconButton(
              key: const Key('title-panel-toggle'),
              tooltip: 'Toggle side panel',
              padding: EdgeInsets.zero,
              constraints: const BoxConstraints.tightFor(width: 32, height: 32),
              onPressed: onTogglePanel,
              icon: const PanelToggleIcon(),
            ),
          ),
        ],
      ],
    ),
  );
}

class WindowIdentity extends StatelessWidget {
  const WindowIdentity({super.key});
  @override
  Widget build(BuildContext context) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      const Icon(Icons.all_inclusive_rounded, size: 18),
      const SizedBox(width: 7),
      Text(
        'Dolores',
        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
          fontWeight: FontWeight.w600,
          height: 1,
          leadingDistribution: TextLeadingDistribution.even,
        ),
      ),
    ],
  );
}

/// A window outline with a narrow left pane, matching the panel it toggles.
class PanelToggleIcon extends StatelessWidget {
  const PanelToggleIcon({super.key});
  @override
  Widget build(BuildContext context) => SizedBox.square(
    dimension: 16,
    child: CustomPaint(
      painter: _PanelPainter(
        IconTheme.of(context).color ?? Theme.of(context).colorScheme.onSurface,
      ),
    ),
  );
}

class _PanelPainter extends CustomPainter {
  final Color color;
  const _PanelPainter(this.color);
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.15;
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        const Rect.fromLTWH(1.5, 2.5, 13, 11),
        const Radius.circular(2.5),
      ),
      paint,
    );
    canvas.drawLine(const Offset(6, 2.5), const Offset(6, 13.5), paint);
  }

  @override
  bool shouldRepaint(_PanelPainter oldDelegate) => color != oldDelegate.color;
}
