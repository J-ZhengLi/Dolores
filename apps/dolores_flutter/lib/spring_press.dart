import 'package:flutter/material.dart';

/// Visual press feedback only; the wrapped control retains input and semantics.
class SpringPress extends StatefulWidget {
  final Widget child;
  const SpringPress({super.key, required this.child});
  @override
  State<SpringPress> createState() => _SpringPressState();
}

class _SpringPressState extends State<SpringPress> {
  bool pressed = false;
  void press(bool value) {
    if (pressed != value) setState(() => pressed = value);
  }

  @override
  Widget build(BuildContext context) {
    final media = MediaQuery.of(context);
    final reduce = media.disableAnimations || media.accessibleNavigation;
    return Listener(
      onPointerDown: (_) => press(true),
      onPointerUp: (_) => press(false),
      onPointerCancel: (_) => press(false),
      child: AnimatedScale(
        scale: pressed && !reduce ? .97 : 1,
        duration: reduce
            ? Duration.zero
            : Duration(milliseconds: pressed ? 140 : 280),
        curve: pressed ? Curves.easeOut : Curves.easeOutBack,
        child: widget.child,
      ),
    );
  }
}
