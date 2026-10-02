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
  @override
  void initState() {
    super.initState();
    tokens = TextEditingController(
      text: '${widget.chat.requestSettings['maxOutputTokens']}',
    );
    timeout = TextEditingController(
      text: '${widget.chat.requestSettings['timeoutSeconds']}',
    );
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
    if (!_form.currentState!.validate()) return;
    setState(() {
      saving = true;
      failure = null;
    });
    try {
      await widget.chat.saveRequestSettings({
        'maxOutputTokens': int.parse(tokens.text.trim()),
        'timeoutSeconds': int.parse(timeout.text.trim()),
      });
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
                  TextFormField(
                    key: const Key('output-token-limit'),
                    controller: tokens,
                    enabled: !saving,
                    keyboardType: TextInputType.number,
                    maxLength: 9,
                    decoration: const InputDecoration(
                      labelText: 'Output token limit',
                      helperText: '1–32,768 tokens · default 2,048',
                      counterText: '',
                    ),
                    validator: (v) => _validate(v, 32768),
                  ),
                  const SizedBox(height: 20),
                  TextFormField(
                    key: const Key('request-timeout'),
                    controller: timeout,
                    enabled: !saving,
                    keyboardType: TextInputType.number,
                    maxLength: 9,
                    decoration: const InputDecoration(
                      labelText: 'Request timeout (seconds)',
                      helperText: '1–900 seconds · default 180',
                      counterText: '',
                    ),
                    validator: (v) => _validate(v, 900),
                  ),
                  const SizedBox(height: 20),
                  Text(
                    'Saved for your next message and future launches. The timeout includes connection time and the full response. Your model may apply a lower output limit.',
                    style: TextStyle(color: p.muted, fontSize: 12),
                  ),
                  const SizedBox(height: 8),
                  TextButton(
                    key: const Key('reset-request-settings'),
                    onPressed: saving
                        ? null
                        : () {
                            tokens.text = '2048';
                            timeout.text = '180';
                            _form.currentState?.validate();
                          },
                    child: const Text('Restore defaults'),
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
