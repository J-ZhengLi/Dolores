import 'package:flutter/material.dart';

import 'chat.dart';
import 'theme.dart';
import 'settings_frame.dart';

import 'dart:convert';

class RequestSettingsDialog extends StatefulWidget {
  final ChatController chat;
  final bool appDefaults;
  const RequestSettingsDialog({
    super.key,
    required this.chat,
    this.appDefaults = false,
  });
  @override
  State<RequestSettingsDialog> createState() => _RequestSettingsDialogState();
}

class _RequestSettingsDialogState extends State<RequestSettingsDialog> {
  late String savedDraft;
  String get draftValue => jsonEncode([
    model,
    endpoint,
    inherited,
    tokens.text,
    timeout.text,
    reasoning,
  ]);
  final _form = GlobalKey<FormState>();
  final _scroll = ScrollController();
  late final TextEditingController tokens, timeout;
  bool saving = false;
  String? failure, notice;
  late String endpoint, model;
  String reasoning = 'providerDefault';
  bool inherited = false;
  Map<String, dynamic> get selectedSettings => widget.appDefaults
      ? widget.chat.defaultRequestSettings
      : model == widget.chat.model
      ? widget.chat.requestSettings
      : (widget.chat.modelRequestSettings[model] as Map?)
                ?.cast<String, dynamic>() ??
            widget.chat.defaultRequestSettings;
  @override
  void initState() {
    super.initState();
    endpoint = widget.chat.baseUrl;
    model = widget.appDefaults ? '' : widget.chat.model;
    tokens = TextEditingController(
      text: selectedSettings['maxOutputTokens']?.toString() ?? '',
    );
    timeout = TextEditingController(
      text: '${selectedSettings['timeoutSeconds']}',
    );
    reasoning = selectedSettings['reasoning'] as String? ?? 'providerDefault';
    savedDraft = draftValue;
  }

  @override
  void dispose() {
    _scroll.dispose();
    tokens.dispose();
    timeout.dispose();
    super.dispose();
  }

  String? _validate(String? value, int max) {
    final n = int.tryParse(value?.trim() ?? '');
    return n == null || n < 1 || n > max
        ? 'Enter a whole number from 1 to $max.'
        : null;
  }

  Future<void> _save() async {
    if (endpoint != widget.chat.baseUrl ||
        (model.isNotEmpty && !widget.chat.enabledModels.contains(model))) {
      setState(
        () => failure = 'The model connection changed. Reload response settings before saving. Your edits are retained.',
      );
      if (_scroll.hasClients) _scroll.jumpTo(0);
      return;
    }
    if (!inherited && !_form.currentState!.validate()) return;
    setState(() {
      saving = true;
      failure = null;
      notice = null;
    });
    try {
      final settings = {
        'maxOutputTokens': int.tryParse(tokens.text.trim()),
        'timeoutSeconds': int.parse(timeout.text.trim()),
        if (reasoning != 'providerDefault') 'reasoning': reasoning,
      };
      if (model.isEmpty) {
        await widget.chat.saveRequestSettings(settings);
      } else {
        await widget.chat.saveModelRequestSettings(
          endpoint,
          model,
          inherited ? null : settings,
        );
      }
      if (mounted) {
        savedDraft = draftValue;
        if (SettingsEmbedding.of(context) == null) {
          Navigator.pop(context);
        } else {
          setState(() => notice = 'Response settings saved.');
        }
      }
    } catch (error) {
      if (mounted) setState(() => failure = error.toString());
      if (mounted && _scroll.hasClients) _scroll.jumpTo(0);
    } finally {
      if (mounted) setState(() => saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    reportSettingsDraft(
      context,
      dirty: () => draftValue != savedDraft,
      save: () async {
        if (saving) return false;
        await _save();
        return draftValue == savedDraft;
      },
    );
    return PopScope(
      canPop: !saving,
      child: SettingsFormFrame(
        title: 'Responses',
        canClose: !saving,
        content: SingleChildScrollView(
          controller: _scroll,
          child: Form(
            key: _form,
            child: Column(
              mainAxisSize: MainAxisSize.min,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (failure != null)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 12),
                    child: Text(failure!, style: TextStyle(color: p.errorText)),
                  ),
                if (model.isNotEmpty) ...[
                  DropdownButtonFormField<String>(
                    key: const Key('generation-model'),
                    initialValue: model,
                    isExpanded: true,
                    decoration: const InputDecoration(labelText: 'Model'),
                    items: {...widget.chat.enabledModels, model}
                        .map(
                          (id) => DropdownMenuItem(
                            value: id,
                            child: Text(id, overflow: TextOverflow.ellipsis),
                          ),
                        )
                        .toList(),
                    onChanged: saving
                        ? null
                        : (value) {
                            setState(() {
                              model = value!;
                              inherited = false;
                              tokens.text =
                                  selectedSettings['maxOutputTokens']
                                      ?.toString() ??
                                  '';
                              timeout.text =
                                  '${selectedSettings['timeoutSeconds']}';
                              reasoning =
                                  selectedSettings['reasoning'] as String? ??
                                  'providerDefault';
                              failure = null;
                            });
                          },
                  ),
                  const SizedBox(height: 20),
                ],
                TextFormField(
                  key: const Key('output-token-limit'),
                  controller: tokens,
                  enabled: !saving && !inherited,
                  keyboardType: TextInputType.number,
                  maxLength: 9,
                  decoration: const InputDecoration(
                    labelText: 'Output tokens (optional)',
                    hintText: 'Provider default',
                    helperText: 'Blank lets the provider choose',
                    counterText: '',
                  ),
                  validator: (v) => inherited || (v?.trim().isEmpty ?? true)
                      ? null
                      : _validate(v, 16777216),
                ),
                const SizedBox(height: 20),
                TextFormField(
                  key: const Key('request-timeout'),
                  controller: timeout,
                  enabled: !saving && !inherited,
                  keyboardType: TextInputType.number,
                  maxLength: 9,
                  decoration: const InputDecoration(
                    labelText: 'Stall timeout (seconds)',
                    helperText: '1–900 seconds · default 180',
                    counterText: '',
                  ),
                  validator: (v) => inherited ? null : _validate(v, 900),
                ),
                const SizedBox(height: 20),
                DropdownButtonFormField<String>(
                  key: ValueKey('reasoning-$model-$reasoning'),
                  initialValue: reasoning,
                  isExpanded: true,
                  decoration: const InputDecoration(
                    labelText: 'Reasoning control',
                  ),
                  items: const [
                    DropdownMenuItem(
                      value: 'providerDefault',
                      child: Text('Provider default'),
                    ),
                    DropdownMenuItem(
                      value: 'deepseekThinkingOff',
                      child: Text('DeepSeek · thinking off'),
                    ),
                    DropdownMenuItem(
                      value: 'glmLow',
                      child: Text('GLM · low effort'),
                    ),
                    DropdownMenuItem(
                      value: 'openaiLow',
                      child: Text('OpenAI · low effort'),
                    ),
                    DropdownMenuItem(
                      value: 'openaiMedium',
                      child: Text('OpenAI · medium effort'),
                    ),
                    DropdownMenuItem(
                      value: 'openaiHigh',
                      child: Text('OpenAI · high effort'),
                    ),
                  ],
                  onChanged: saving || inherited
                      ? null
                      : (value) => setState(() => reasoning = value!),
                ),
                const SizedBox(height: 12),
                SettingsDetails(
                  title: 'How response limits apply',
                  children: [
                    Text(
                      'Stall timeout waits for new model data; active responses and review can take longer. Scope overrides take precedence. Output may include thinking tokens. Reasoning controls depend on provider support.',
                      style: TextStyle(color: p.muted, fontSize: 12),
                    ),
                  ],
                ),
                const SizedBox(height: 8),
                TextButton(
                  key: const Key('reset-request-settings'),
                  onPressed: saving
                      ? null
                      : () {
                          setState(() {
                            inherited = model.isNotEmpty;
                            final defaults = widget.chat.defaultRequestSettings;
                            tokens.text =
                                defaults['maxOutputTokens']?.toString() ?? '';
                            timeout.text = '${defaults['timeoutSeconds']}';
                            reasoning =
                                defaults['reasoning'] as String? ??
                                'providerDefault';
                          });
                          _form.currentState?.validate();
                        },
                  child: const Text('Restore defaults'),
                ),
                if (inherited)
                  const Text(
                    'Save to remove this model’s override and use the application defaults.',
                  ),
                if (notice != null) Text(notice!),
              ],
            ),
          ),
        ),
        actions: [
          if (SettingsEmbedding.of(context) != null)
            TextButton(
              onPressed: saving
                  ? null
                  : () => setState(() {
                      endpoint = widget.chat.baseUrl;
                      model = widget.appDefaults ? '' : widget.chat.model;
                      inherited = false;
                      tokens.text =
                          selectedSettings['maxOutputTokens']?.toString() ?? '';
                      timeout.text = '${selectedSettings['timeoutSeconds']}';
                      reasoning =
                          selectedSettings['reasoning'] as String? ??
                          'providerDefault';
                      failure = null;
                      notice = 'Response settings reloaded.';
                      savedDraft = draftValue;
                    }),
              child: const Text('Reload response settings'),
            ),
          if (SettingsEmbedding.of(context) == null)
            TextButton(
              onPressed: saving ? null : () => Navigator.pop(context),
              child: const Text('Cancel'),
            ),
          FilledButton(
            key: const Key('save-request-settings'),
            onPressed: saving ? null : _save,
            child: Text(saving ? 'Saving…' : 'Save'),
          ),
        ],
      ),
    );
  }
}
