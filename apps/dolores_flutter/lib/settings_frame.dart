import 'package:flutter/material.dart';

/// Allows existing settings editors to share one window without nested dialogs.
class SettingsEmbedding extends InheritedWidget {
  final ValueNotifier<bool> pending;
  final ModalRoute<dynamic>? route;
  const SettingsEmbedding({
    super.key,
    required this.pending,
    required this.route,
    required super.child,
  });
  static SettingsEmbedding? of(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<SettingsEmbedding>();
    return scope?.route == ModalRoute.of(context) ? scope : null;
  }

  @override
  bool updateShouldNotify(SettingsEmbedding oldWidget) =>
      pending != oldWidget.pending;
}

class EmbeddedSettingsFrame extends StatefulWidget {
  final String title, subtitle;
  final Widget child;
  final bool canClose;
  final ValueNotifier<bool> pending;
  const EmbeddedSettingsFrame({
    super.key,
    required this.title,
    this.subtitle = '',
    required this.child,
    required this.canClose,
    required this.pending,
  });
  @override
  State<EmbeddedSettingsFrame> createState() => _EmbeddedSettingsFrameState();
}

class _EmbeddedSettingsFrameState extends State<EmbeddedSettingsFrame> {
  @override
  void initState() {
    super.initState();
    report();
  }

  @override
  void didUpdateWidget(EmbeddedSettingsFrame oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.canClose != widget.canClose) report();
  }

  void report() {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) widget.pending.value = !widget.canClose;
    });
  }

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    children: [
      Padding(
        padding: const EdgeInsets.fromLTRB(20, 20, 20, 12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(widget.title, style: Theme.of(context).textTheme.titleLarge),
            if (widget.subtitle.isNotEmpty) ...[
              const SizedBox(height: 4),
              Text(
                widget.subtitle,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ],
        ),
      ),
      Expanded(child: widget.child),
    ],
  );
}

/// Native dialog outside Settings; a scrolling editor inside Settings.
class SettingsFormFrame extends StatelessWidget {
  final String title;
  final Widget content;
  final List<Widget> actions;
  final bool canClose;
  const SettingsFormFrame({
    super.key,
    required this.title,
    required this.content,
    required this.actions,
    this.canClose = true,
  });
  @override
  Widget build(BuildContext context) {
    final embedding = SettingsEmbedding.of(context);
    if (embedding == null) {
      return AlertDialog(
        title: Text(title),
        content: SizedBox(width: 420, child: content),
        actions: actions,
      );
    }
    return EmbeddedSettingsFrame(
      title: title,
      canClose: canClose,
      pending: embedding.pending,
      child: Column(
        children: [
          Expanded(
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 20),
              child: content,
            ),
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Wrap(spacing: 8, runSpacing: 8, children: actions),
          ),
        ],
      ),
    );
  }
}
