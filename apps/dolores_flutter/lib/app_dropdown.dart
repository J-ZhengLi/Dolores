import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// A select field whose rounded menu opens below its trigger and fits the screen.
class AppDropdown<T> extends StatefulWidget {
  final T? initialValue;
  final List<DropdownMenuItem<T>>? items;
  final ValueChanged<T?>? onChanged;
  final InputDecoration decoration;
  final Widget? hint;
  const AppDropdown({
    super.key,
    this.initialValue,
    required this.items,
    required this.onChanged,
    this.decoration = const InputDecoration(),
    this.hint,
  });

  @override
  State<AppDropdown<T>> createState() => _AppDropdownState<T>();
}

class _AppDropdownState<T> extends State<AppDropdown<T>> {
  final focus = FocusNode();
  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => LayoutBuilder(
    builder: (context, constraints) {
      final width = constraints.hasBoundedWidth ? constraints.maxWidth : 240.0;
      final selected = widget.items
          ?.where((item) => item.value == widget.initialValue)
          .firstOrNull;
      final enabled =
          widget.onChanged != null && (widget.items?.isNotEmpty ?? false);
      return MenuAnchor(
        childFocusNode: focus,
        useRootOverlay: true,
        alignmentOffset: const Offset(0, 4),
        crossAxisUnconstrained: false,
        style: MenuStyle(
          minimumSize: WidgetStatePropertyAll(Size(width, 0)),
          maximumSize: WidgetStatePropertyAll(Size(width, 320)),
        ),
        menuChildren: [
          for (final item in widget.items ?? <DropdownMenuItem<T>>[])
            MenuItemButton(
              onPressed: !enabled || !item.enabled
                  ? null
                  : () => widget.onChanged!(item.value),
              trailingIcon: item.value == widget.initialValue
                  ? const Icon(Icons.check, size: 16)
                  : null,
              child: DefaultTextStyle.merge(
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                child: item.child,
              ),
            ),
        ],
        builder: (context, controller, _) => CallbackShortcuts(
          bindings: {
            const SingleActivator(LogicalKeyboardKey.arrowDown): () {
              if (enabled) controller.open();
            },
          },
          child: Semantics(
            button: true,
            enabled: enabled,
            child: InkWell(
              focusNode: focus,
              borderRadius: BorderRadius.circular(10),
              onTap: !enabled
                  ? null
                  : () => controller.isOpen
                        ? controller.close()
                        : controller.open(),
              child: InputDecorator(
                decoration: widget.decoration.copyWith(enabled: enabled),
                isEmpty: selected == null && widget.hint == null,
                child: Row(
                  children: [
                    Expanded(
                      child: DefaultTextStyle.merge(
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                        child:
                            selected?.child ??
                            widget.hint ??
                            const Text('Choose…'),
                      ),
                    ),
                    const SizedBox(width: 8),
                    Icon(
                      controller.isOpen ? Icons.expand_less : Icons.expand_more,
                      size: 18,
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      );
    },
  );
}
