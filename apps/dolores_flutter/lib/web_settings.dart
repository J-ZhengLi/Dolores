import 'package:flutter/material.dart';

import 'chat.dart';
import 'inspector.dart';

Future<void> showWebSettings(BuildContext context, ChatController chat) =>
    showDialog<void>(
      context: context,
      builder: (_) => WebSettingsInspector(chat: chat),
    );

class WebSettingsInspector extends StatefulWidget {
  final ChatController chat;
  const WebSettingsInspector({super.key, required this.chat});
  @override
  State<WebSettingsInspector> createState() => _WebSettingsState();
}

class _WebSettingsState extends State<WebSettingsInspector> {
  final endpoint = TextEditingController(), apiKey = TextEditingController();
  final scroll = ScrollController();
  Map? report;
  bool enabled = true, pending = false, clearKey = false;
  String provider = 'mwmbl';
  String? error, notice;
  @override
  void initState() {
    super.initState();
    load();
  }

  @override
  void dispose() {
    endpoint.dispose();
    apiKey.dispose();
    scroll.dispose();
    super.dispose();
  }

  Future<void> load({bool keepDraft = false}) async {
    setState(() => pending = true);
    try {
      final value =
          await widget.chat.bridge.call({'command': 'webSettings'}) as Map;
      if (!mounted) return;
      setState(() {
        report = value;
        error = null;
        if (!keepDraft) {
          enabled = value['enabled'] as bool;
          provider = value['provider'] as String;
          endpoint.text = value['endpoint'] as String? ?? '';
          apiKey.clear();
          clearKey = false;
        }
      });
    } catch (failure) {
      if (mounted) setState(() => error = failure.toString());
    } finally {
      if (mounted) setState(() => pending = false);
    }
  }

  Future<void> save() async {
    setState(() {
      pending = true;
      error = null;
      notice = null;
    });
    try {
      final value = await widget.chat.bridge.call({
        'command': 'saveWebSettings',
        'revision': report!['revision'],
        'enabled': enabled,
        'provider': provider,
        'endpoint': provider == 'searxng' ? endpoint.text.trim() : null,
        'apiKey': provider != 'brave' || apiKey.text.isEmpty
            ? null
            : apiKey.text,
        'clearKey': clearKey,
      }) as Map;
      if (!mounted) return;
      setState(() {
        report = value;
        notice = value['notice'] as String?;
        apiKey.clear();
        clearKey = false;
      });
      widget.chat.invalidateContextPreview();
    } catch (failure) {
      if (mounted) setState(() => error = failure.toString());
    } finally {
      if (mounted) {
        setState(() => pending = false);
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted && scroll.hasClients) scroll.jumpTo(0);
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: widget.chat,
    builder: (context, _) {
      final locked = pending || widget.chat.busy || widget.chat.changing;
      return PopScope(
        canPop: !pending,
        child: InspectorFrame(
          title: 'Web search',
          subtitle: 'Global connection · working chats only',
          canClose: !pending,
          child: Column(
            children: [
              Expanded(
                child: SingleChildScrollView(
                  controller: scroll,
                  padding: const EdgeInsets.all(20),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      if (error != null) SelectableText(error!),
                      if (notice != null) Text(notice!),
                      if (report == null && pending)
                        const LinearProgressIndicator(),
                      const Text(
                        'Default search works without an account or API key. Queries go to Mwmbl’s public index; its coverage is smaller than commercial engines. You can choose another connection below.',
                      ),
                      const SizedBox(height: 12),
                      SwitchListTile(
                        key: const Key('web-enabled'),
                        contentPadding: EdgeInsets.zero,
                        title: const Text(
                          'Enable web search and public page reading',
                        ),
                        value: enabled,
                        onChanged: locked
                            ? null
                            : (value) => setState(() => enabled = value),
                      ),
                      DropdownButtonFormField<String>(
                        key: const Key('web-provider'),
                        isExpanded: true,
                        initialValue: provider,
                        decoration: const InputDecoration(
                          labelText: 'Search provider',
                        ),
                        items: const [
                          DropdownMenuItem(
                            value: 'mwmbl',
                            child: Text('Default (Mwmbl · no key)'),
                          ),
                          DropdownMenuItem(
                            value: 'brave',
                            child: Text('Brave Search API'),
                          ),
                          DropdownMenuItem(
                            value: 'searxng',
                            child: Text('Custom SearXNG'),
                          ),
                        ],
                        onChanged: locked
                            ? null
                            : (value) => setState(() => provider = value!),
                      ),
                      if (provider == 'searxng') ...[
                        const SizedBox(height: 12),
                        TextField(
                          key: const Key('web-endpoint'),
                          controller: endpoint,
                          enabled: !locked,
                          decoration: const InputDecoration(
                            labelText: 'Public HTTPS search endpoint',
                            hintText: 'https://search.example.org/search',
                          ),
                        ),
                        const Text(
                          'The instance must enable JSON search. No private-network endpoint, login or embedded key is supported.',
                        ),
                      ],
                      if (provider == 'brave') ...[
                        const SizedBox(height: 12),
                        TextField(
                          key: const Key('web-api-key'),
                          controller: apiKey,
                          enabled: !locked && !clearKey,
                          obscureText: true,
                          autocorrect: false,
                          enableSuggestions: false,
                          decoration: InputDecoration(
                            labelText: report?['hasSavedKey'] == true
                                ? 'Replace API key (blank keeps saved key)'
                                : 'API key',
                          ),
                        ),
                        const Text(
                          'Stored in the OS credential vault. Brave queries may consume your plan’s quota; Dolores does not purchase a subscription or increase limits.',
                        ),
                      ],
                      if (report?['hasSavedKey'] == true)
                        CheckboxListTile(
                          key: const Key('web-clear-key'),
                          contentPadding: EdgeInsets.zero,
                          title: const Text('Remove saved Brave key'),
                          value: clearKey,
                          onChanged: locked
                              ? null
                              : (value) => setState(() {
                                  clearKey = value!;
                                  if (clearKey) apiKey.clear();
                                }),
                        ),
                      const SizedBox(height: 16),
                      const Text(
                        'Web operations share the exact query or URL with the displayed service. Results are shared with your chat model and retained in run evidence. No files, cookies or model keys are sent by this adapter. Task permissions and limits still apply.',
                      ),
                      const SizedBox(height: 12),
                      const Text(
                        'One query returns up to five sources. Page reads accept public HTTPS HTML/plain text only: 20 seconds, 256 KiB download and 8 KiB excerpt. No redirects, scripts, login or automatic provider switching. If your routing proxy supplies synthetic DNS addresses, the public hostname is resolved through Cloudflare DNS; local/private addresses remain blocked. If blocked or empty, refine the query, supply a source URL or choose another provider.',
                      ),
                      if (report?['cleanupPending'] == true)
                        const Text(
                          'Old key cleanup may be pending. Unlock secure storage and Save again to retry.',
                        ),
                    ],
                  ),
                ),
              ),
              Padding(
                padding: const EdgeInsets.all(16),
                child: Wrap(
                  spacing: 8,
                  children: [
                    TextButton(
                      key: const Key('web-refresh'),
                      onPressed: pending ? null : () => load(keepDraft: true),
                      child: const Text('Refresh (keep edits)'),
                    ),
                    FilledButton(
                      key: const Key('web-save'),
                      onPressed: locked || report == null ? null : save,
                      child: Text(pending ? 'Saving…' : 'Save'),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
      );
    },
  );
}
