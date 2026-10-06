import 'dart:convert';

import 'package:flutter/material.dart';

import 'chat.dart';
import 'model_settings.dart';
import 'settings_frame.dart';

class ModelsSettings extends StatefulWidget {
  final ChatController chat;
  final bool initialDetail;
  const ModelsSettings({
    super.key,
    required this.chat,
    this.initialDetail = false,
  });
  @override
  State<ModelsSettings> createState() => _ModelsSettingsState();
}

class _ModelsSettingsState extends State<ModelsSettings> {
  bool connection = false;
  @override
  void initState() {
    super.initState();
    if (widget.initialDetail && widget.chat.model.isNotEmpty) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) details(widget.chat.model);
      });
    }
  }

  Future<void> details(String model) => showDialog<void>(
    context: context,
    barrierDismissible: false,
    builder: (_) => ModelDetailsEditor(chat: widget.chat, model: model),
  );
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.chat,
    builder: (context, _) {
      if (connection || !widget.chat.configured) {
        return Column(
          children: [
            if (widget.chat.configured)
              Align(
                alignment: Alignment.centerLeft,
                child: TextButton.icon(
                  onPressed: () async {
                    final draft = SettingsEmbedding.of(context)?.draft;
                    if (draft?.dirty == true &&
                        !await resolveSettingsDraft(
                          context,
                          save: () => draft!.save!(),
                        )) {
                      return;
                    }
                    if (mounted) setState(() => connection = false);
                  },
                  icon: const Icon(Icons.arrow_back, size: 18),
                  label: const Text('Models'),
                ),
              ),
            Expanded(
              child: ConnectionDialog(
                chat: widget.chat,
                setupOnly: true,
                onSaved: () => setState(() => connection = false),
              ),
            ),
          ],
        );
      }
      reportSettingsDraft(context, dirty: () => false, save: () async => true);
      return EmbeddedSettingsFrame(
        title: 'Models',
        subtitle: 'Choose a model to customize',
        canClose: true,
        pending: SettingsEmbedding.of(context)!.pending,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            for (final model in widget.chat.enabledModels)
              ListTile(
                key: Key('model-details-$model'),
                title: Text(model),
                subtitle: Text(
                  '${((widget.chat.modelContexts[model] ?? 131072) / 1024).round()}K context · ${widget.chat.imageModels.contains(model) ? 'Images enabled' : 'Image support not set'}',
                ),
                trailing: const Icon(Icons.chevron_right),
                onTap: () => details(model),
              ),
            const SizedBox(height: 8),
            TextButton.icon(
              key: const Key('manage-model-connection'),
              onPressed: () => setState(() => connection = true),
              icon: const Icon(Icons.add),
              label: const Text('Connect or choose models'),
            ),
          ],
        ),
      );
    },
  );
}

class ModelDetailsEditor extends StatefulWidget {
  final ChatController chat;
  final String model;
  const ModelDetailsEditor({
    super.key,
    required this.chat,
    required this.model,
  });
  @override
  State<ModelDetailsEditor> createState() => _ModelDetailsEditorState();
}

class _ModelDetailsEditorState extends State<ModelDetailsEditor> {
  final contextTokens = TextEditingController(),
      output = TextEditingController(),
      timeout = TextEditingController();
  late final endpoint = widget.chat.baseUrl;
  Map<String, dynamic>? expected;
  String? savedDraft, error, notice;
  bool images = false, custom = false, pending = false;
  String reasoning = 'providerDefault';
  String get draft => jsonEncode([
    contextTokens.text,
    output.text,
    timeout.text,
    images,
    custom,
    reasoning,
  ]);
  bool get dirty => savedDraft != null && savedDraft != draft;
  Map<String, dynamic> get preferences => {
    'baseUrl': endpoint,
    'model': widget.model,
  };
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    contextTokens.dispose();
    output.dispose();
    timeout.dispose();
    super.dispose();
  }

  Future<void> load({bool keepEdits = false}) async {
    setState(() {
      pending = true;
      error = null;
    });
    try {
      final value = (await widget.chat.bridge.call({
        'command': 'modelDetails',
        'preferences': preferences,
      }) as Map).cast<String, dynamic>();
      if (!mounted) return;
      setState(() {
        expected = value;
        if (!keepEdits) {
          contextTokens.text = value['contextWindowTokens']?.toString() ?? '';
          images = value['imageInput'] == true;
          custom = value['requestSettings'] != null;
          final request =
              (value['requestSettings'] as Map?) ??
              widget.chat.defaultRequestSettings;
          output.text = request['maxOutputTokens']?.toString() ?? '';
          timeout.text = '${request['timeoutSeconds']}';
          reasoning = request['reasoning'] as String? ?? 'providerDefault';
          savedDraft = draft;
        }
      });
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<bool> save() async {
    if (pending || expected == null || widget.chat.busy) return false;
    final capacity = int.tryParse(contextTokens.text.trim()),
        tokens = int.tryParse(output.text.trim()),
        seconds = int.tryParse(timeout.text);
    if ((contextTokens.text.trim().isNotEmpty &&
            (capacity == null || capacity < 1024 || capacity > 16777216)) ||
        (custom &&
            ((output.text.trim().isNotEmpty &&
                    (tokens == null || tokens < 1 || tokens > 16777216)) ||
                seconds == null ||
                seconds < 1 ||
                seconds > 900))) {
      setState(
        () => error = 'Use whole numbers: context 1024–16777216 (or blank), output 1–16777216 (or blank), timeout 1–900.',
      );
      return false;
    }
    setState(() {
      pending = true;
      error = null;
      notice = null;
    });
    try {
      final value = (await widget.chat.bridge.call({
        'command': 'setModelDetails',
        'preferences': preferences,
        'expected': expected,
        'details': {
          'contextWindowTokens': capacity,
          'imageInput': images,
          'requestSettings': custom
              ? {
                  'maxOutputTokens': tokens,
                  'timeoutSeconds': seconds,
                  'reasoning': reasoning,
                }
              : null,
        },
      }) as Map).cast<String, dynamic>();
      if (!mounted) return true;
      setState(() {
        expected = value;
        savedDraft = draft;
        notice = 'Saved';
      });
      // The acknowledgement is durable. A failed refresh must not report a failed save.
      try {
        await widget.chat.refresh();
        widget.chat.invalidateContext();
      } catch (_) {
        if (mounted) {
          setState(
            () => notice =
                'Saved. Reopen settings to refresh the displayed model list.',
          );
        }
      }
      return true;
    } catch (e) {
      if (mounted) setState(() => error = '$e');
      return false;
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> close() async {
    if (pending) return;
    if (dirty && !await resolveSettingsDraft(context, save: save)) return;
    if (mounted) Navigator.pop(context);
  }

  @override
  Widget build(BuildContext context) => CloseOnEscape(
    onClose: close,
    child: PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) close();
      },
      child: Dialog(
        insetPadding: const EdgeInsets.all(16),
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: 560, maxHeight: 620),
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(20, 12, 8, 8),
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        widget.model,
                        overflow: TextOverflow.ellipsis,
                        style: Theme.of(context).textTheme.titleMedium,
                      ),
                    ),
                    IconButton(
                      onPressed: pending ? null : close,
                      tooltip: 'Close model details',
                      icon: const Icon(Icons.close),
                    ),
                  ],
                ),
              ),
              Expanded(
                child: ListView(
                  padding: const EdgeInsets.all(20),
                  children: [
                    if (error != null)
                      Text(
                        error!,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.error,
                        ),
                      ),
                    if (notice != null) Text(notice!),
                    TextField(
                      key: const Key('model-detail-context'),
                      controller: contextTokens,
                      enabled: !pending,
                      keyboardType: TextInputType.number,
                      decoration: const InputDecoration(
                        labelText: 'Context window (tokens)',
                        hintText: '131072',
                        helperText: 'Blank uses 128K',
                      ),
                    ),
                    SwitchListTile(
                      key: const Key('model-detail-images'),
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Image input'),
                      subtitle: Text(
                        images ? 'Enabled in model configuration' : 'Unknown or disabled. Enable only if your provider supports images.',
                      ),
                      value: images,
                      onChanged: pending
                          ? null
                          : (v) => setState(() => images = v),
                    ),
                    const Text(
                      'Window sharing can check image support before sending a private screenshot.',
                    ),
                    ExpansionTile(
                      key: const Key('model-detail-advanced'),
                      title: const Text('Response settings'),
                      subtitle: Text(
                        custom ? 'Custom for this model' : 'Using app defaults',
                      ),
                      children: [
                        SwitchListTile(
                          title: const Text('Customize responses'),
                          value: custom,
                          onChanged: pending
                              ? null
                              : (v) => setState(() => custom = v),
                        ),
                        if (custom) ...[
                          TextField(
                            key: const Key('model-detail-output'),
                            controller: output,
                            enabled: !pending,
                            keyboardType: TextInputType.number,
                            decoration: const InputDecoration(
                              labelText: 'Output tokens (optional)',
                              hintText: 'Provider default',
                              helperText: 'Blank lets the provider choose',
                            ),
                          ),
                          const SizedBox(height: 12),
                          TextField(
                            key: const Key('model-detail-timeout'),
                            controller: timeout,
                            enabled: !pending,
                            keyboardType: TextInputType.number,
                            decoration: const InputDecoration(
                              labelText: 'Stall timeout (seconds)',
                              helperText: 'Time without new model data; review has no timeout',
                            ),
                          ),
                          const SizedBox(height: 12),
                          DropdownButtonFormField<String>(
                            initialValue: reasoning,
                            isExpanded: true,
                            decoration: const InputDecoration(
                              labelText: 'Reasoning',
                            ),
                            items: [
                              for (final r in [
                                'providerDefault',
                                'deepseekThinkingOff',
                                'glmLow',
                                'openaiLow',
                                'openaiMedium',
                                'openaiHigh',
                              ])
                                DropdownMenuItem(
                                  value: r,
                                  child: Text(switch (r) {
                                    'providerDefault' => 'Provider default',
                                    'deepseekThinkingOff' =>
                                      'DeepSeek thinking off',
                                    'openaiLow' => 'OpenAI low',
                                    'glmLow' => 'GLM low',
                                    'openaiMedium' => 'OpenAI medium',
                                    _ => 'OpenAI high',
                                  }),
                                ),
                            ],
                            onChanged: pending
                                ? null
                                : (v) => setState(() => reasoning = v!),
                          ),
                        ],
                        const Padding(
                          padding: EdgeInsets.all(8),
                          child: Text(
                            'Project/chat output and timeout overrides take precedence. Changes apply to the next task.',
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(12),
                child: Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      onPressed: pending ? null : () => load(keepEdits: true),
                      child: const Text('Refresh · keep edits'),
                    ),
                    FilledButton(
                      key: const Key('save-model-details'),
                      onPressed: pending || expected == null ? null : save,
                      child: Text(pending ? 'Saving…' : 'Save'),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    ),
  );
}
