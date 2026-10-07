import 'dart:async';

import 'package:flutter/material.dart';

import 'language_host.dart';

Future<void> showLanguageSetup(BuildContext context, LanguageHost host) =>
    showDialog<void>(
      context: context,
      barrierDismissible: false,
      builder: (_) => LanguageSetup(host: host),
    );

class LanguageSetup extends StatefulWidget {
  final LanguageHost host;
  const LanguageSetup({super.key, required this.host});
  @override
  State<LanguageSetup> createState() => _LanguageSetupState();
}

class _LanguageSetupState extends State<LanguageSetup> {
  String? id, error;
  Map? info;
  Map progress = {'state': 'idle'};
  bool starting = false;
  @override
  void initState() {
    super.initState();
    unawaited(load());
  }

  Future<void> load() async {
    try {
      final v = await widget.host.call({'action': 'installInfo'});
      if (mounted) setState(() => info = v);
    } catch (e) {
      if (mounted) setState(() => error = '$e');
    }
  }

  Future<void> install(String language) async {
    setState(() {
      starting = true;
      error = null;
    });
    try {
      await widget.host.stop();
      widget.host.used = true;
      final job = await widget.host.call({
        'action': 'install',
        'language': language,
      });
      if (!mounted) return;
      setState(() => id = job['id']);
      final until = DateTime.now().add(const Duration(minutes: 5));
      while (mounted && DateTime.now().isBefore(until)) {
        final v = await widget.host.call({'action': 'installPoll', 'id': id});
        if (!mounted) return;
        setState(() => progress = v);
        if (v['state'] == 'done' || v['state'] == 'failed') {
          setState(() {
            error = v['error'];
            id = null;
          });
          return;
        }
        await Future<void>.delayed(const Duration(milliseconds: 300));
      }
      if (id != null) {
        await widget.host.call({'action': 'cancelInstall', 'id': id});
      }
      throw StateError(
        'Installation timed out. Previous tools remain; inspect setup and retry.',
      );
    } catch (e) {
      if (mounted) {
        setState(() {
          error = '$e';
          id = null;
        });
      }
    } finally {
      if (mounted) setState(() => starting = false);
    }
  }

  @override
  void dispose() {
    if (id != null) {
      unawaited(
        widget.host
            .call({'action': 'cancelInstall', 'id': id})
            .catchError((Object _) => null),
      );
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: id == null && !starting,
    child: AlertDialog(
      title: const Text('Language tools'),
      content: SizedBox(
        width: 540,
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Install verified tools into Dolores’s private profile. Node.js is required for TypeScript. Language servers run with your account’s permissions. Files stay editable if setup fails.',
            ),
            const SizedBox(height: 12),
            if (info != null) Text(info!['note'].toString()),
            const SizedBox(height: 12),
            Text(
              'Status: ${progress['state']}${progress['bytes'] == null ? '' : ' · ${(progress['bytes'] as num) / 1048576} MiB'}',
            ),
            if (id != null || starting) const LinearProgressIndicator(),
            if (error != null) SelectableText(error!),
            const SizedBox(height: 12),
            Wrap(
              spacing: 8,
              children: [
                FilledButton(
                  onPressed: info == null || starting
                      ? null
                      : () => unawaited(install('typescript')),
                  child: Text(
                    'Install TypeScript ${info?['typescript']?['compiler'] ?? ''}',
                  ),
                ),
                OutlinedButton(
                  onPressed: info == null || starting
                      ? null
                      : () => unawaited(install('rust')),
                  child: Text(
                    'Install Rust tools ${info?['rust']?['version'] ?? ''}',
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
      actions: [
        if (id != null)
          TextButton(
            onPressed: () => unawaited(
              widget.host.call({'action': 'cancelInstall', 'id': id}),
            ),
            child: const Text('Cancel installation'),
          ),
        TextButton(
          onPressed: id != null || starting
              ? null
              : () => Navigator.pop(context),
          child: const Text('Close'),
        ),
      ],
    ),
  );
}
