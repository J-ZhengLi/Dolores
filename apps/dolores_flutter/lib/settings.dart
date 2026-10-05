import 'package:flutter/material.dart';

import 'chat.dart';
import 'dolores_settings.dart';
import 'mcp.dart';
import 'model_settings.dart';
import 'request_settings.dart';
import 'settings_frame.dart';
import 'skills.dart';
import 'task_permissions.dart';
import 'theme.dart';
import 'web_settings.dart';
import 'browser_settings.dart';
import 'knowledge.dart';

enum SettingsCategory {
  appearance,
  models,
  personalization,
  memory,
  web,
  browser,
  tools,
  skills,
  permissions,
  limits,
}

enum ModelsPage { connection, responses, overrides }

Future<void> showSettings(
  BuildContext context,
  ChatController chat, {
  SettingsCategory initial = SettingsCategory.models,
  ModelsPage modelsPage = ModelsPage.connection,
}) async {
  await showDialog<void>(
    context: context,
    barrierDismissible: false,
    builder: (_) =>
        SettingsWindow(chat: chat, initial: initial, modelsPage: modelsPage),
  );
  chat.invalidateContext();
}

class SettingsWindow extends StatefulWidget {
  final ChatController chat;
  final SettingsCategory initial;
  final ModelsPage modelsPage;
  const SettingsWindow({
    super.key,
    required this.chat,
    this.initial = SettingsCategory.models,
    this.modelsPage = ModelsPage.connection,
  });
  @override
  State<SettingsWindow> createState() => _SettingsWindowState();
}

class _SettingsWindowState extends State<SettingsWindow> {
  late SettingsCategory selected = widget.initial;
  late ModelsPage modelPage = widget.modelsPage;
  final panels = <String, Widget>{};
  final locks = <String, ValueNotifier<bool>>{};
  String get panelKey => selected == SettingsCategory.models
      ? 'models-${modelPage.name}'
      : selected.name;
  bool get pending => locks.values.any((lock) => lock.value);
  static const labels = {
    SettingsCategory.appearance: ('Appearance', Icons.contrast),
    SettingsCategory.models: ('Models', Icons.auto_awesome_outlined),
    SettingsCategory.personalization: ('Personalization', Icons.tune),
    SettingsCategory.memory: ('Memory', Icons.bookmarks_outlined),
    SettingsCategory.web: ('Web search', Icons.travel_explore),
    SettingsCategory.browser: ('Browser', Icons.web_outlined),
    SettingsCategory.tools: ('External tools', Icons.extension_outlined),
    SettingsCategory.skills: ('Skills', Icons.auto_stories_outlined),
    SettingsCategory.permissions: ('Permissions', Icons.shield_outlined),
    SettingsCategory.limits: ('Task limits', Icons.timer_outlined),
  };
  @override
  void dispose() {
    for (final lock in locks.values) {
      lock.dispose();
    }
    super.dispose();
  }

  void changed() {
    if (mounted) setState(() {});
  }

  Widget unavailable(String message) =>
      Padding(padding: const EdgeInsets.all(24), child: Text(message));
  Widget editor() => switch (selected) {
    SettingsCategory.skills =>
      widget.chat.session == null
          ? unavailable(
              'Select a saved chat to manage global and project skills.',
            )
          : widget.chat.busy
          ? unavailable(
              'Finish or stop the current response before editing skills.',
            )
          : SkillsInspector(
              bridge: widget.chat.bridge,
              session: widget.chat.session!,
              hasProject: widget.chat.workspaceRoot != null,
            ),
    SettingsCategory.appearance => AppearanceSettings(chat: widget.chat),
    SettingsCategory.models => switch (modelPage) {
      ModelsPage.connection => ConnectionDialog(chat: widget.chat),
      ModelsPage.responses => RequestSettingsDialog(chat: widget.chat),
      ModelsPage.overrides => DoloresSettingsInspector(
        chat: widget.chat,
        group: SettingsGroup.generation,
      ),
    },
    SettingsCategory.personalization => DoloresSettingsInspector(
      chat: widget.chat,
      group: SettingsGroup.personalization,
    ),
    SettingsCategory.limits => DoloresSettingsInspector(
      chat: widget.chat,
      group: SettingsGroup.task,
    ),
    SettingsCategory.memory =>
      widget.chat.busy
          ? unavailable(
              'Finish or stop the current response before editing memory.',
            )
          : MemorySettings(chat: widget.chat),
    SettingsCategory.web => WebSettingsInspector(chat: widget.chat),
    SettingsCategory.browser => BrowserSettingsInspector(chat: widget.chat),
    SettingsCategory.tools =>
      widget.chat.session == null || widget.chat.workspaceRoot == null
          ? unavailable(
              'Open a project or send the first message in a temporary workspace to configure external tools for its working folder.',
            )
          : widget.chat.busy
          ? unavailable(
              'Finish or stop the current response before configuring external tools.',
            )
          : McpInspector(
              bridge: widget.chat.bridge,
              session: widget.chat.session!,
            ),
    SettingsCategory.permissions =>
      widget.chat.session == null || widget.chat.workspaceRoot == null
          ? unavailable(
              'Select a working chat to configure its permissions. Side chats have no tool access.',
            )
          : TaskPermissionsInspector(chat: widget.chat),
  };
  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    if (!panels.containsKey(panelKey)) {
      final lock = ValueNotifier(false)..addListener(changed);
      locks[panelKey] = lock;
      panels[panelKey] = SettingsEmbedding(
        pending: lock,
        route: ModalRoute.of(context),
        child: editor(),
      );
    }
    return PopScope(
      canPop: !pending,
      child: Dialog(
        key: const Key('settings-window'),
        insetPadding: const EdgeInsets.all(16),
        backgroundColor: p.bg,
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 960, maxHeight: 720),
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 12, 12, 12),
                child: Row(
                  children: [
                    const Expanded(
                      child: Text(
                        'Settings',
                        style: TextStyle(
                          fontSize: 20,
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    ),
                    TextButton(
                      key: const Key('close-settings'),
                      onPressed: pending ? null : () => Navigator.pop(context),
                      child: const Text('Close'),
                    ),
                  ],
                ),
              ),
              Divider(height: 1, color: p.border),
              Expanded(
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    final compact = constraints.maxWidth < 640;
                    final content = Column(
                      children: [
                        if (selected == SettingsCategory.models && !compact)
                          Padding(
                            padding: const EdgeInsets.fromLTRB(20, 12, 20, 0),
                            child: Wrap(
                              spacing: 8,
                              runSpacing: 8,
                              children: [
                                for (final page in ModelsPage.values)
                                  ChoiceChip(
                                    key: Key('models-${page.name}'),
                                    label: Text(switch (page) {
                                      ModelsPage.connection =>
                                        'Connection & models',
                                      ModelsPage.responses => 'Responses',
                                      ModelsPage.overrides => 'Scope overrides',
                                    }),
                                    selected: modelPage == page,
                                    onSelected: pending
                                        ? null
                                        : (_) =>
                                              setState(() => modelPage = page),
                                  ),
                              ],
                            ),
                          ),
                        Expanded(
                          child: IndexedStack(
                            sizing: StackFit.expand,
                            index: panels.keys.toList().indexOf(panelKey),
                            children: panels.entries
                                .map(
                                  (entry) => KeyedSubtree(
                                    key: ValueKey(entry.key),
                                    child: entry.value,
                                  ),
                                )
                                .toList(),
                          ),
                        ),
                      ],
                    );
                    if (compact) {
                      return Column(
                        children: [
                          Padding(
                            padding: const EdgeInsets.all(12),
                            child: Row(
                              children: [
                                Expanded(
                                  child:
                                      DropdownButtonFormField<SettingsCategory>(
                                        key: const Key('settings-category'),
                                        initialValue: selected,
                                        isExpanded: true,
                                        decoration: const InputDecoration(
                                          labelText: 'Category',
                                        ),
                                        items: [
                                          for (final category
                                              in SettingsCategory.values)
                                            DropdownMenuItem(
                                              value: category,
                                              child: Text(labels[category]!.$1),
                                            ),
                                        ],
                                        onChanged: pending
                                            ? null
                                            : (value) => setState(
                                                () => selected = value!,
                                              ),
                                      ),
                                ),
                                if (selected == SettingsCategory.models) ...[
                                  const SizedBox(width: 12),
                                  Expanded(
                                    child: DropdownButtonFormField<ModelsPage>(
                                      key: const Key('models-page'),
                                      initialValue: modelPage,
                                      isExpanded: true,
                                      decoration: const InputDecoration(
                                        labelText: 'Model settings',
                                      ),
                                      items: [
                                        for (final page in ModelsPage.values)
                                          DropdownMenuItem(
                                            value: page,
                                            child: Text(switch (page) {
                                              ModelsPage.connection =>
                                                'Connection',
                                              ModelsPage.responses =>
                                                'Responses',
                                              ModelsPage.overrides =>
                                                'Scope overrides',
                                            }, overflow: TextOverflow.ellipsis),
                                          ),
                                      ],
                                      onChanged: pending
                                          ? null
                                          : (value) => setState(
                                              () => modelPage = value!,
                                            ),
                                    ),
                                  ),
                                ],
                              ],
                            ),
                          ),
                          Expanded(child: content),
                        ],
                      );
                    }
                    return Row(
                      children: [
                        Container(
                          width: 200,
                          color: p.sidebar,
                          child: ListView(
                            padding: const EdgeInsets.all(12),
                            children: [
                              for (final category in SettingsCategory.values)
                                Padding(
                                  padding: const EdgeInsets.only(bottom: 4),
                                  child: TextButton.icon(
                                    key: Key('settings-${category.name}'),
                                    onPressed: pending
                                        ? null
                                        : () => setState(
                                            () => selected = category,
                                          ),
                                    style: TextButton.styleFrom(
                                      alignment: Alignment.centerLeft,
                                      foregroundColor: p.text,
                                      backgroundColor: selected == category
                                          ? p.soft
                                          : null,
                                      padding: const EdgeInsets.symmetric(
                                        horizontal: 12,
                                        vertical: 16,
                                      ),
                                      shape: RoundedRectangleBorder(
                                        borderRadius: BorderRadius.circular(10),
                                      ),
                                    ),
                                    icon: Icon(labels[category]!.$2, size: 18),
                                    label: Text(labels[category]!.$1),
                                  ),
                                ),
                            ],
                          ),
                        ),
                        VerticalDivider(width: 1, color: p.border),
                        Expanded(child: content),
                      ],
                    );
                  },
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class AppearanceSettings extends StatefulWidget {
  final ChatController chat;
  const AppearanceSettings({super.key, required this.chat});
  @override
  State<AppearanceSettings> createState() => _AppearanceSettingsState();
}

class _AppearanceSettingsState extends State<AppearanceSettings> {
  bool pending = false;
  String? error;
  Future<void> save(String appearance) async {
    if (pending || widget.chat.appearance == appearance) return;
    setState(() {
      pending = true;
      error = null;
    });
    try {
      await widget.chat.saveAppearance(appearance);
    } catch (failure) {
      if (mounted) setState(() => error = '$failure');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  @override
  Widget build(BuildContext context) => EmbeddedSettingsFrame(
    title: 'Appearance',
    subtitle: 'Choose how Dolores looks',
    pending: SettingsEmbedding.of(context)!.pending,
    canClose: !pending,
    child: ListView(
      padding: const EdgeInsets.all(20),
      children: [
        if (error != null)
          Padding(
            padding: const EdgeInsets.only(bottom: 12),
            child: Text('$error Your previous theme is retained. Try again.'),
          ),
        const Text('Theme', style: TextStyle(fontWeight: FontWeight.w600)),
        const SizedBox(height: 12),
        Wrap(
          spacing: 12,
          runSpacing: 12,
          children: [
            for (final mode in ['system', 'light', 'dark'])
              SizedBox(
                width: 144,
                child: OutlinedButton(
                  key: Key('theme-$mode'),
                  onPressed: pending ? null : () => save(mode),
                  style: OutlinedButton.styleFrom(
                    padding: const EdgeInsets.all(18),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    side: BorderSide(
                      color: widget.chat.appearance == mode
                          ? Theme.of(context).colorScheme.primary
                          : Theme.of(context).dividerColor,
                      width: widget.chat.appearance == mode ? 2 : 1,
                    ),
                  ),
                  child: Column(
                    children: [
                      ExcludeSemantics(
                        child: Container(
                          height: 64,
                          decoration: BoxDecoration(
                            borderRadius: BorderRadius.circular(8),
                            color: mode == 'light'
                                ? const Color(0xfff5f5f5)
                                : const Color(0xff191b20),
                          ),
                          child: Row(
                            children: [
                              Container(
                                width: 25,
                                decoration: BoxDecoration(
                                  color: mode == 'light'
                                      ? const Color(0xffe4e5e9)
                                      : const Color(0xff2a2d36),
                                  borderRadius: const BorderRadius.horizontal(
                                    left: Radius.circular(8),
                                  ),
                                ),
                              ),
                              const Expanded(
                                child: Padding(
                                  padding: EdgeInsets.all(8),
                                  child: Column(
                                    mainAxisAlignment: MainAxisAlignment.center,
                                    children: [
                                      LinearProgressIndicator(
                                        value: 0.65,
                                        color: Color(0xff9bb0ff),
                                        backgroundColor: Color(0xff50545d),
                                      ),
                                      SizedBox(height: 8),
                                      LinearProgressIndicator(
                                        value: 0.4,
                                        color: Color(0xff9bb0ff),
                                        backgroundColor: Color(0xff50545d),
                                      ),
                                    ],
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ),
                      const SizedBox(height: 10),
                      Text(switch (mode) {
                        'system' => 'System',
                        'light' => 'Light',
                        _ => 'Dark',
                      }),
                    ],
                  ),
                ),
              ),
          ],
        ),
        const SizedBox(height: 16),
        const Text(
          'System follows your device. Changes apply immediately and are saved for the next launch.',
        ),
        if (pending) const LinearProgressIndicator(),
      ],
    ),
  );
}
