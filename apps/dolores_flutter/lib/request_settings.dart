import 'package:flutter/material.dart';

import 'chat.dart';
import 'theme.dart';

class RequestSettingsDialog extends StatefulWidget {
  final ChatController chat;
  const RequestSettingsDialog({super.key, required this.chat});
  @override
  State<RequestSettingsDialog> createState() => _RequestSettingsDialogState();
}

class _RequestSettingsDialogState extends State<RequestSettingsDialog> {
  final _form = GlobalKey<FormState>();
  late final TextEditingController tokens, timeout;
  bool saving = false;
  String? failure;
  late String endpoint, model;
  String reasoning = 'providerDefault';
  bool inherited = false;
  Map<String, dynamic> get selectedSettings => model == widget.chat.model
      ? widget.chat.requestSettings
      : (widget.chat.modelRequestSettings[model] as Map?)
                ?.cast<String, dynamic>() ??
            widget.chat.defaultRequestSettings;
  @override
  void initState() {
    super.initState();
    endpoint = widget.chat.baseUrl;
    model = widget.chat.model;
    tokens = TextEditingController(
      text: '${widget.chat.requestSettings['maxOutputTokens']}',
    );
    timeout = TextEditingController(
      text: '${widget.chat.requestSettings['timeoutSeconds']}',
    );
    reasoning = selectedSettings['reasoning'] as String? ?? 'providerDefault';
  }

  @override
  void dispose() {
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
    if (!inherited && !_form.currentState!.validate()) return;
    setState(() {
      saving = true;
      failure = null;
    });
    try {
      final settings = {
        'maxOutputTokens': int.parse(tokens.text.trim()),
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
      if (mounted) Navigator.pop(context);
    } catch (error) {
      if (mounted) setState(() => failure = error.toString());
    } finally {
      if (mounted) setState(() => saving = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return PopScope(
      canPop: !saving,
      child: AlertDialog(
        title: const Text('Request settings'),
        content: SizedBox(
          width: 400,
          child: SingleChildScrollView(
            child: Form(
              key: _form,
              child: Column(
                mainAxisSize: MainAxisSize.min,
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  if (model.isNotEmpty) ...[
                    DropdownButtonFormField<String>(
                      key: const Key('generation-model'),
                      initialValue: model,
                      isExpanded: true,
                      decoration: const InputDecoration(labelText: 'Model'),
                      items: widget.chat.enabledModels
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
                                    '${selectedSettings['maxOutputTokens']}';
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
                      labelText: 'Output token limit',
                      helperText: '1–32,768 tokens · default 2,048',
                      counterText: '',
                    ),
                    validator: (v) => inherited ? null : _validate(v, 32768),
                  ),
                  const SizedBox(height: 20),
                  TextFormField(
                    key: const Key('request-timeout'),
                    controller: timeout,
                    enabled: !saving && !inherited,
                    keyboardType: TextInputType.number,
                    maxLength: 9,
                    decoration: const InputDecoration(
                      labelText: 'Request timeout (seconds)',
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
                  Text(
                    'Saved for this model and endpoint. Output includes reasoning tokens when reported. The timeout includes the full request and tool review. Choose a reasoning control only if your provider supports it; Provider default sends no override.',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  const SizedBox(height: 8),
                  TextButton(
                    key: const Key('reset-request-settings'),
                    onPressed: saving
                        ? null
                        : () {
                            setState(() {
                              inherited = model.isNotEmpty;
                              final defaults =
                                  widget.chat.defaultRequestSettings;
                              tokens.text = '${defaults['maxOutputTokens']}';
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
                  if (failure != null)
                    Padding(
                      padding: const EdgeInsets.only(top: 12),
                      child: Text(
                        failure!,
                        style: TextStyle(color: p.errorText),
                      ),
                    ),
                ],
              ),
            ),
          ),
        ),
        actions: [
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
