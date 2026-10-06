import 'learning.dart';
import 'tools_home.dart';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'dolores_settings.dart';
import 'mcp.dart';
import 'models.dart';
import 'request_settings.dart';
import 'settings_frame.dart';
import 'tool_trials.dart';
import 'task_permissions.dart';
import 'theme.dart';
import 'web_settings.dart';
import 'browser_settings.dart';
import 'knowledge.dart';
import 'mods.dart';
import 'desktop_settings.dart';
import 'comparison.dart';
import 'capabilities.dart';
import 'attachment_storage.dart';
import 'advanced_home.dart';
import 'native_repairs.dart';
import 'experimental_settings.dart';

enum SettingsCategory {
  experimental,
  advancedHome,
  nativeRepairs,
  appearance,
  models,
  personalization,
  memory,
  toolHome,
  web,
  browser,
  desktop,
  tools,
  skills,
  skillTesting,
  skillLearning,
  mods,
  permissions,
  limits,
  comparisons,
  capabilities,
  storage,
}

enum ModelsPage { connection, responses, overrides, defaults }

Future<void> showSettings(
  BuildContext context,
  ChatController chat, {
  SettingsCategory initial = SettingsCategory.appearance,
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

enum SettingsSection {
  general,
  models,
  personalization,
  memory,
  tools,
  advanced,
}

SettingsSection sectionFor(SettingsCategory category, ModelsPage page) =>
    switch (category) {
      SettingsCategory.appearance ||
      SettingsCategory.experimental => SettingsSection.general,
      SettingsCategory.models =>
        page == ModelsPage.overrides || page == ModelsPage.defaults
            ? SettingsSection.advanced
            : SettingsSection.models,
      SettingsCategory.personalization => SettingsSection.personalization,
      SettingsCategory.memory => SettingsSection.memory,
      SettingsCategory.advancedHome ||
      SettingsCategory.nativeRepairs ||
      SettingsCategory.mods ||
      SettingsCategory.limits ||
      SettingsCategory.skillTesting ||
      SettingsCategory.skillLearning ||
      SettingsCategory.comparisons ||
      SettingsCategory.capabilities ||
      SettingsCategory.storage => SettingsSection.advanced,
      _ => SettingsSection.tools,
    };

class _SettingsWindowState extends State<SettingsWindow> {
  late SettingsCategory selected = widget.initial;
  late ModelsPage modelPage = widget.modelsPage;
  final panels = <String, Widget>{};
  final locks = <String, ValueNotifier<bool>>{};
  final drafts = <String, SettingsDraft>{};
  final search = TextEditingController();
  final editorStackKey = GlobalKey();
  bool closing = false;
  SettingsSection get section => sectionFor(selected, modelPage);
  String get panelKey => selected == SettingsCategory.models
      ? 'models-${modelPage.name}'
      : selected.name;
  bool get pending => closing || locks.values.any((lock) => lock.value);
  static const labels = {
    SettingsSection.general: ('General', Icons.contrast),
    SettingsSection.models: ('Models', Icons.auto_awesome_outlined),
    SettingsSection.personalization: ('Personalization', Icons.tune),
    SettingsSection.memory: ('Memory', Icons.bookmarks_outlined),
    SettingsSection.tools: ('Tools', Icons.extension_outlined),
    SettingsSection.advanced: ('Advanced', Icons.settings_outlined),
  };
  static const destinations = <(String, SettingsCategory, ModelsPage, String)>[
    (
      'Advanced overview',
      SettingsCategory.advancedHome,
      ModelsPage.connection,
      'tuning testing troubleshooting',
    ),
    (
      'Compare instructions',
      SettingsCategory.comparisons,
      ModelsPage.connection,
      'evaluation baseline candidate snapshots',
    ),
    (
      'Dolores capabilities',
      SettingsCategory.capabilities,
      ModelsPage.connection,
      'diagnostics harness source registry tools',
    ),
    (
      'Attachment storage',
      SettingsCategory.storage,
      ModelsPage.connection,
      'clean unused snapshots attachments cleanup',
    ),
    (
      'Response defaults',
      SettingsCategory.models,
      ModelsPage.defaults,
      'request default generation output timeout reasoning',
    ),
    (
      'Theme',
      SettingsCategory.appearance,
      ModelsPage.connection,
      'appearance system light dark',
    ),
    (
      'Experimental',
      SettingsCategory.experimental,
      ModelsPage.connection,
      'multiple window keep awake prevent windows locked',
    ),
    (
      'Connection & models',
      SettingsCategory.models,
      ModelsPage.connection,
      'api provider key context image vision',
    ),
    (
      'Responses',
      SettingsCategory.models,
      ModelsPage.responses,
      'request settings output timeout reasoning',
    ),
    (
      'Explanation style',
      SettingsCategory.personalization,
      ModelsPage.connection,
      'personalization assumptions discuss brief',
    ),
    (
      'Memory',
      SettingsCategory.memory,
      ModelsPage.connection,
      'preferences project facts remember sharing',
    ),
    (
      'Tools overview',
      SettingsCategory.toolHome,
      ModelsPage.connection,
      'readiness built-in',
    ),
    (
      'Web search',
      SettingsCategory.web,
      ModelsPage.connection,
      'brave searxng default search',
    ),
    (
      'Browser',
      SettingsCategory.browser,
      ModelsPage.connection,
      'browser setup capture',
    ),
    (
      'Computer use',
      SettingsCategory.desktop,
      ModelsPage.connection,
      'desktop screenshot window access recovery',
    ),
    (
      'Connections',
      SettingsCategory.tools,
      ModelsPage.connection,
      'external tools mcp import credentials',
    ),
    (
      'Skills',
      SettingsCategory.skills,
      ModelsPage.connection,
      'global project import create draft',
    ),
    (
      'Skill testing',
      SettingsCategory.skillTesting,
      ModelsPage.connection,
      'tool trials baseline candidate evaluation',
    ),
    (
      'Learning experiments',
      SettingsCategory.skillLearning,
      ModelsPage.connection,
      'reflection activation policy pause suggestions',
    ),
    (
      'Access',
      SettingsCategory.permissions,
      ModelsPage.connection,
      'permissions review automatic full access grants',
    ),
    (
      'Task limits',
      SettingsCategory.limits,
      ModelsPage.connection,
      'execution model calls tool segments deadline',
    ),
    (
      'Scope overrides',
      SettingsCategory.models,
      ModelsPage.overrides,
      'project chat generation inheritance',
    ),
    (
      'Harness extensions',
      SettingsCategory.mods,
      ModelsPage.connection,
      'mods source tests restore',
    ),
    (
      'Native repairs',
      SettingsCategory.nativeRepairs,
      ModelsPage.connection,
      'native qualified build install restart restore repair',
    ),
  ];
  void navigate(
    SettingsCategory category, [
    ModelsPage page = ModelsPage.connection,
  ]) {
    setState(() {
      selected = category;
      modelPage = page;
      search.clear();
    });
  }

  void selectSection(SettingsSection value) {
    final target = destinations.firstWhere(
      (d) =>
          sectionFor(d.$2, d.$3) == value &&
          (value != SettingsSection.advanced ||
              d.$2 == SettingsCategory.advancedHome),
    );
    navigate(target.$2, target.$3);
  }

  @override
  void dispose() {
    search.dispose();
    for (final lock in locks.values) {
      lock.dispose();
    }
    super.dispose();
  }

  void changed() {
    if (mounted) setState(() {});
  }

  Future<void> close() async {
    if (pending) return;
    final unsaved = drafts.entries.where((e) => e.value.dirty).toList();
    if (unsaved.isNotEmpty) {
      final leave = await resolveSettingsDraft(
        context,
        save: () async {
          setState(() => closing = true);
          try {
            for (final entry in unsaved) {
              if (entry.value.save == null || !await entry.value.save!()) {
                if (mounted) {
                  setState(() {
                    final target = destinations.firstWhere(
                      (d) =>
                          (d.$2 == SettingsCategory.models
                              ? 'models-${d.$3.name}'
                              : d.$2.name) ==
                          entry.key,
                    );
                    selected = target.$2;
                    modelPage = target.$3;
                    search.clear();
                  });
                }
                return false;
              }
            }
            return true;
          } finally {
            if (mounted) setState(() => closing = false);
          }
        },
      );
      if (!leave || !mounted) return;
    }
    if (mounted) Navigator.pop(context);
  }

  Widget unavailable(String message) =>
      Padding(padding: const EdgeInsets.all(24), child: Text(message));
  Widget editor() => switch (selected) {
    SettingsCategory.experimental => ExperimentalSettings(chat: widget.chat),
    SettingsCategory.nativeRepairs => NativeRepairs(
      chat: widget.chat,
      hasUnsavedSettings: () => drafts.values.any((draft) => draft.dirty),
    ),
    SettingsCategory.advancedHome => AdvancedHome(
      open: (category, page) => navigate(
        SettingsCategory.values.byName(category),
        ModelsPage.values.byName(page),
      ),
    ),
    SettingsCategory.comparisons =>
      widget.chat.session == null
          ? unavailable('Start a chat before comparing instruction snapshots.')
          : ComparisonInspector(chat: widget.chat),
    SettingsCategory.capabilities => CapabilitiesInspector(chat: widget.chat),
    SettingsCategory.storage => AttachmentStorage(chat: widget.chat),
    SettingsCategory.mods =>
      widget.chat.busy
          ? unavailable(
              'Finish or stop the current response before managing mods.',
            )
          : ModsInspector(chat: widget.chat),
    SettingsCategory.skills =>
      widget.chat.busy
          ? unavailable('Stop the current response before editing skills.')
          : SkillSettings(chat: widget.chat),
    SettingsCategory.skillTesting => ToolTrialsInspector(chat: widget.chat),
    SettingsCategory.skillLearning => LearningInspector(chat: widget.chat),
    SettingsCategory.appearance => AppearanceSettings(chat: widget.chat),
    SettingsCategory.models => switch (modelPage) {
      ModelsPage.connection => ModelsSettings(chat: widget.chat),
      ModelsPage.responses => ModelsSettings(
        chat: widget.chat,
        initialDetail: true,
      ),
      ModelsPage.defaults => RequestSettingsDialog(
        chat: widget.chat,
        appDefaults: true,
      ),
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
    SettingsCategory.toolHome => ToolsHome(
      chat: widget.chat,
      open: (name) => navigate(SettingsCategory.values.byName(name)),
    ),
    SettingsCategory.web => WebSettingsInspector(chat: widget.chat),
    SettingsCategory.browser => BrowserSettingsInspector(chat: widget.chat),
    SettingsCategory.desktop => DesktopSettingsInspector(chat: widget.chat),
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
      final draft = SettingsDraft();
      drafts[panelKey] = draft;
      panels[panelKey] = SettingsEmbedding(
        pending: lock,
        draft: draft,
        route: ModalRoute.of(context),
        child: editor(),
      );
    }
    final query = search.text.trim().toLowerCase();
    final matches = destinations
        .where((d) => '${d.$1} ${d.$4}'.toLowerCase().contains(query))
        .toList();
    final pages = destinations
        .where(
          (d) =>
              sectionFor(d.$2, d.$3) == section && d.$3 != ModelsPage.responses,
        )
        .toList();
    return CloseOnEscape(
      onClose: close,
      child: PopScope(
        canPop: false,
        onPopInvokedWithResult: (didPop, _) {
          if (!didPop) close();
        },
        child: Dialog(
          key: const Key('settings-window'),
          insetPadding: const EdgeInsets.all(16),
          backgroundColor: p.bg,
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 960, maxHeight: 720),
            child: Column(
              children: [
                Padding(
                  padding: const EdgeInsets.fromLTRB(20, 12, 12, 8),
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
                        onPressed: pending ? null : close,
                        child: const Text('Close'),
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
                  child: TextField(
                    key: const Key('settings-search'),
                    controller: search,
                    enabled: !pending,
                    onChanged: (_) => changed(),
                    decoration: const InputDecoration(
                      hintText: 'Search settings',
                      prefixIcon: Icon(Icons.search, size: 18),
                      isDense: true,
                    ),
                  ),
                ),
                Divider(height: 1, color: p.border),
                Expanded(
                  child: LayoutBuilder(
                    builder: (context, constraints) {
                      final compact = constraints.maxWidth < 640;
                      final content = Stack(
                        children: [
                          Column(
                            children: [
                              if (pages.length > 1)
                                Padding(
                                  padding: const EdgeInsets.fromLTRB(
                                    16,
                                    8,
                                    16,
                                    0,
                                  ),
                                  child: DropdownButtonFormField<int>(
                                    key: ValueKey(
                                      'settings-page-${section.name}-${selected.name}-${modelPage.name}',
                                    ),
                                    initialValue: pages.indexWhere(
                                      (d) =>
                                          d.$2 == selected &&
                                          (selected !=
                                                  SettingsCategory.models ||
                                              d.$3 == modelPage),
                                    ),
                                    isExpanded: true,
                                    decoration: const InputDecoration(
                                      isDense: true,
                                    ),
                                    items: [
                                      for (var i = 0; i < pages.length; i++)
                                        DropdownMenuItem(
                                          value: i,
                                          child: Text(pages[i].$1),
                                        ),
                                    ],
                                    onChanged: pending
                                        ? null
                                        : (i) => navigate(
                                            pages[i!].$2,
                                            pages[i].$3,
                                          ),
                                  ),
                                ),
                              Expanded(
                                key: const ValueKey('cached-settings-editors'),
                                child: IndexedStack(
                                  key: editorStackKey,
                                  sizing: StackFit.expand,
                                  index: panels.keys.toList().indexOf(panelKey),
                                  children: panels.entries
                                      .map(
                                        (e) => KeyedSubtree(
                                          key: ValueKey(e.key),
                                          child: e.value,
                                        ),
                                      )
                                      .toList(),
                                ),
                              ),
                            ],
                          ),
                          if (query.isNotEmpty)
                            Positioned.fill(
                              child: Material(
                                color: p.bg,
                                child: ListView(
                                  children: [
                                    if (matches.isEmpty)
                                      const ListTile(
                                        title: Text('No matching settings'),
                                      ),
                                    for (final d in matches)
                                      ListTile(
                                        key: Key(
                                          'setting-result-${d.$2.name}-${d.$3.name}',
                                        ),
                                        title: Text(d.$1),
                                        subtitle: Text(
                                          labels[sectionFor(d.$2, d.$3)]!.$1,
                                        ),
                                        onTap: pending
                                            ? null
                                            : () => navigate(d.$2, d.$3),
                                      ),
                                  ],
                                ),
                              ),
                            ),
                        ],
                      );
                      if (compact) {
                        return Column(
                          children: [
                            Padding(
                              padding: const EdgeInsets.all(12),
                              child: DropdownButtonFormField<SettingsSection>(
                                key: ValueKey(
                                  'settings-category-${section.name}',
                                ),
                                initialValue: section,
                                isExpanded: true,
                                decoration: const InputDecoration(
                                  isDense: true,
                                ),
                                items: [
                                  for (final value in SettingsSection.values)
                                    DropdownMenuItem(
                                      value: value,
                                      child: Text(labels[value]!.$1),
                                    ),
                                ],
                                onChanged: pending
                                    ? null
                                    : (value) => selectSection(value!),
                              ),
                            ),
                            Expanded(child: content),
                          ],
                        );
                      }
                      return Row(
                        children: [
                          Container(
                            width: 180,
                            color: p.sidebar,
                            child: ListView(
                              padding: const EdgeInsets.all(12),
                              children: [
                                for (final value in SettingsSection.values)
                                  Padding(
                                    padding: const EdgeInsets.only(bottom: 4),
                                    child: TextButton.icon(
                                      key: Key('settings-${value.name}'),
                                      onPressed: pending
                                          ? null
                                          : () => selectSection(value),
                                      style: TextButton.styleFrom(
                                        alignment: Alignment.centerLeft,
                                        foregroundColor: p.text,
                                        backgroundColor: section == value
                                            ? p.soft
                                            : null,
                                        padding: const EdgeInsets.symmetric(
                                          horizontal: 12,
                                          vertical: 16,
                                        ),
                                      ),
                                      icon: Icon(labels[value]!.$2, size: 18),
                                      label: Text(labels[value]!.$1),
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
    title: 'General',
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
        LayoutBuilder(
          builder: (context, constraints) => Wrap(
            spacing: 12,
            runSpacing: 12,
            children: [
              for (final mode in ['system', 'light', 'dark'])
                SizedBox(
                  width: ((constraints.maxWidth - 24) / 3).clamp(84, 144),
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
                          child: SizedBox(
                            height: 64,
                            width: double.infinity,
                            child: CustomPaint(
                              painter: ThemePreviewPainter(mode),
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
        ),
        const SizedBox(height: 16),
        const Text(
          'System follows your device. Theme changes save immediately.',
        ),
        if (pending) const LinearProgressIndicator(),
      ],
    ),
  );
}

/// One miniature window, split by palette rather than duplicating its layout.
class ThemePreviewPainter extends CustomPainter {
  final String mode;
  const ThemePreviewPainter(this.mode);
  @override
  void paint(Canvas canvas, Size size) {
    final bounds = Offset.zero & size;
    canvas.save();
    canvas.clipRRect(RRect.fromRectAndRadius(bounds, const Radius.circular(8)));
    void scene(bool dark) {
      final p = Palette(dark);
      canvas.drawRect(bounds, Paint()..color = p.bg);
      canvas.drawRect(
        Rect.fromLTWH(0, 0, size.width * .23, size.height),
        Paint()..color = p.sidebar,
      );
      for (final line in [(0.36, .27, .53), (0.36, .44, .32)]) {
        canvas.drawRRect(
          RRect.fromRectAndRadius(
            Rect.fromLTWH(
              size.width * line.$1,
              size.height * line.$2,
              size.width * line.$3,
              4,
            ),
            const Radius.circular(2),
          ),
          Paint()..color = p.muted,
        );
      }
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(
            size.width * .53,
            size.height * .66,
            size.width * .36,
            10,
          ),
          const Radius.circular(4),
        ),
        Paint()..color = p.soft,
      );
    }

    scene(mode == 'dark');
    if (mode == 'system') {
      canvas.save();
      canvas.clipRect(
        Rect.fromLTWH(size.width * .5, 0, size.width * .5, size.height),
      );
      scene(true);
      canvas.restore();
    }
    canvas.restore();
  }

  @override
  bool shouldRepaint(ThemePreviewPainter oldDelegate) =>
      oldDelegate.mode != mode;
}
