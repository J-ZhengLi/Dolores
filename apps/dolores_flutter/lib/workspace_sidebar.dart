import 'package:flutter/material.dart';

/// Desktop navigation rows grow with text scaling instead of clipping labels.
class WorkspaceSidebarRow extends StatelessWidget {
  final Widget child;
  final VoidCallback? onTap, onDoubleTap;
  final bool selected;
  final double indent, minHeight;
  const WorkspaceSidebarRow({
    super.key,
    required this.child,
    this.onTap,
    this.onDoubleTap,
    this.selected = false,
    this.indent = 0,
    this.minHeight = 26,
  });

  @override
  Widget build(BuildContext context) => Material(
    color: selected
        ? Theme.of(context).colorScheme.primary.withValues(alpha: .10)
        : Colors.transparent,
    child: InkWell(
      onTap: onTap,
      onDoubleTap: onDoubleTap,
      child: ConstrainedBox(
        constraints: BoxConstraints(minHeight: minHeight),
        child: Padding(
          padding: EdgeInsets.only(left: 12 + indent, right: 8),
          child: DefaultTextStyle.merge(
            style: const TextStyle(fontSize: 13, height: 1.25),
            child: child,
          ),
        ),
      ),
    ),
  );
}

class WorkspaceSidebarButton extends StatelessWidget {
  final String tooltip;
  final IconData icon;
  final VoidCallback? onPressed;
  const WorkspaceSidebarButton({
    super.key,
    required this.tooltip,
    required this.icon,
    required this.onPressed,
  });

  @override
  Widget build(BuildContext context) => IconButton(
    tooltip: tooltip,
    onPressed: onPressed,
    icon: Icon(icon, size: 16),
    padding: EdgeInsets.zero,
    constraints: const BoxConstraints.tightFor(width: 28, height: 26),
    style: IconButton.styleFrom(
      tapTargetSize: MaterialTapTargetSize.shrinkWrap,
    ),
  );
}
