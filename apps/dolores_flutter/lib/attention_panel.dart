import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import 'attention_host.dart';
import 'help_tip.dart';

class AttentionButton extends StatefulWidget {
  final AttentionHost host;
  const AttentionButton({super.key, required this.host});
  @override
  State<AttentionButton> createState() => _AttentionButtonState();
}

class _AttentionButtonState extends State<AttentionButton> {
  final controller = MenuController();
  final focus = FocusNode();
  AttentionHost get host => widget.host;
  @override
  void dispose() {
    focus.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: host,
    builder: (context, _) => MenuAnchor(
      useRootOverlay: true,
      controller: controller,
      childFocusNode: focus,
      alignmentOffset: const Offset(-332, 6),
      menuChildren: [
        Builder(
          builder: (context) => SizedBox(
            width: (MediaQuery.sizeOf(context).width - 32).clamp(280, 380),
            height: math.min(
              host.items.isEmpty
                  ? 260
                  : math.min(260.0 + 72 * host.items.length, 480.0),
              MediaQuery.sizeOf(context).height - 84,
            ),
            child: CallbackShortcuts(
              bindings: {
                const SingleActivator(LogicalKeyboardKey.escape):
                    controller.close,
              },
              child: Focus(
                autofocus: true,
                child: AttentionPanel(host: host, onClose: controller.close),
              ),
            ),
          ),
        ),
      ],
      builder: (context, controller, _) => IconButton(
        key: const Key('task-attention'),
        focusNode: focus,
        tooltip: 'Attention',
        constraints: const BoxConstraints.tightFor(width: 46, height: 32),
        padding: EdgeInsets.zero,
        onPressed: () =>
            controller.isOpen ? controller.close() : controller.open(),
        icon: Badge(
          isLabelVisible: host.unread > 0 || host.error != null,
          label: Text(host.error != null ? '!' : '${host.unread}'),
          child: const Icon(Icons.notifications_none, size: 17),
        ),
      ),
    ),
  );
}

class AttentionPanel extends StatefulWidget {
  final AttentionHost host;
  final VoidCallback? onClose;
  const AttentionPanel({super.key, required this.host, this.onClose});
  @override
  State<AttentionPanel> createState() => _AttentionPanelState();
}

class _AttentionPanelState extends State<AttentionPanel> {
  AttentionHost get host => widget.host;
  @override
  void initState() {
    super.initState();
    host.addListener(changed);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (mounted) host.refresh();
    });
  }

  void changed() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    host.removeListener(changed);
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.all(16),
    child: Column(
      children: [
        Row(
          children: [
            const Expanded(
              child: HelpLabel(
                help: 'Task updates and requests appear here. Opening an update takes you to the task; it does not approve or answer it.',
                child: Text(
                  'Attention',
                  style: TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
                ),
              ),
            ),
            IconButton(
              tooltip: 'Close attention',
              onPressed: widget.onClose ?? () => Navigator.pop(context),
              icon: const Icon(Icons.close, size: 18),
            ),
          ],
        ),
        Expanded(
          child: ListView(
            primary: false,
            children: [
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: const HelpLabel(
                  help: 'Notifications show only task status, without chat content or file names. Dolores must be open or in the tray. Windows may hide notifications; delivery does not mean they were read.',
                  child: Text('Windows notifications'),
                ),
                value: host.preferences['enabled'] == true,
                onChanged: host.pending
                    ? null
                    : (v) => host.save({...host.preferences, 'enabled': v}),
              ),
              if (host.preferences['enabled'] == true)
                ExpansionTile(
                  title: const Text('Notify about'),
                  children: [
                    for (final kind in [
                      'completed',
                      'failed',
                      'approval',
                      'question',
                    ])
                      CheckboxListTile(
                        title: Text(attentionLabel(kind)),
                        value: host.preferences[kind] == true,
                        onChanged: host.pending
                            ? null
                            : (v) => host.save({...host.preferences, kind: v}),
                      ),
                  ],
                ),
              if (!host.supported)
                const Text(
                  'Windows notifications unavailable. Updates still appear here.',
                ),
              if (host.omitted > 0)
                Text('${host.omitted} older updates are in task history.'),
              if (host.items.isEmpty && !host.pending)
                const Text('You’re all caught up.'),
              for (final n in host.items)
                Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: ListTile(
                    key: Key('attention-open-${n['id']}'),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(10),
                    ),
                    contentPadding: const EdgeInsets.symmetric(horizontal: 8),
                    leading: Icon(
                      n['seen'] == true ? Icons.task_alt : Icons.circle,
                      size: n['seen'] == true ? 18 : 8,
                    ),
                    title: Text(
                      n['title'] as String? ??
                          attentionLabel(n['kind'] as String),
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                    subtitle: n['title'] == null
                        ? null
                        : Text(attentionLabel(n['kind'] as String)),
                    trailing: n['seen'] == true
                        ? null
                        : IconButton(
                            tooltip: 'Mark seen',
                            icon: const Icon(Icons.done, size: 18),
                            onPressed: host.pending
                                ? null
                                : () => host.seen(n['id'] as String),
                          ),
                    onTap: host.pending
                        ? null
                        : () async {
                            if (widget.onClose != null) {
                              widget.onClose!();
                            } else {
                              Navigator.pop(context);
                            }
                            await host.open(n['id'] as String);
                          },
                  ),
                ),
            ],
          ),
        ),
        if (host.error != null)
          ConstrainedBox(
            constraints: const BoxConstraints(maxHeight: 96),
            child: SingleChildScrollView(
              primary: false,
              child: SelectableText(
                host.error!,
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ),
          ),
        if (host.pending) const LinearProgressIndicator(),
        Wrap(
          spacing: 8,
          children: [
            TextButton(
              onPressed: host.pending ? null : () => host.refresh(),
              child: const Text('Refresh'),
            ),
            if (host.cursor != null)
              TextButton(
                onPressed: host.pending ? null : () => host.refresh(next: true),
                child: const Text('Older'),
              ),
            if (host.deferred != null)
              TextButton(
                onPressed: host.pending
                    ? null
                    : () async {
                        if (widget.onClose != null) {
                          widget.onClose!();
                        } else {
                          Navigator.pop(context);
                        }
                        await host.open(host.deferred!);
                      },
                child: const Text('Open pending task'),
              ),
          ],
        ),
      ],
    ),
  );
}
