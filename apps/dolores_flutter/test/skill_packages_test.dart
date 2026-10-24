import 'dart:async';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/inspector.dart';
import 'package:dolores_flutter/skill_packages.dart';
import 'package:dolores_flutter/skills.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

const first =
    'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa';
const second =
    'bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb';
Map<String, dynamic> installed(String id) => {
  'document': {
    'name': 'review',
    'description': 'Review public code',
    'text': 'Read references/check.md. Do not run scripts during installation.',
  },
  'package': {
    'id': id,
    'contentDigest': id,
    'root': 'C:/fixture/skill-packages/$id/payload',
    'manifest': {
      'schema': 1,
      'version': id == first ? '1.0' : '2.0',
      'origin': 'https://example.invalid/public',
      'dependencies': [
        {'name': 'python', 'requirement': '3.x (inspection only)'},
      ],
    },
    'resources': [
      {'path': 'SKILL.md', 'bytes': 93, 'digest': id},
      {'path': 'references/check.md', 'bytes': 32, 'digest': id},
      {'path': 'assets/image.bin', 'bytes': 3, 'digest': id},
    ],
    'directories': ['references', 'assets', 'assets/empty'],
  },
  'installedAt': 1,
};

class PackagesBridge implements ChatBridge {
  final calls = <Map<String, dynamic>>[];
  final registered = <String, Set<String>>{'project': {}, 'global': {}};
  final versions = <String, List<Map>>{'project': [], 'global': []};
  String selected = first;
  int tokens = 0;
  bool failInstall = false, stale = false, damaged = false, failExport = false;
  bool failForget = false;
  Completer<void>? pending;
  Map<String, dynamic> active(String scope) => {
    'name': 'review',
    'enabled': true,
    'revision': versions[scope]!.length,
    'versions': versions[scope],
  };
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    calls.add(command);
    if (command['command'] == 'projectSkills') return {'items': []};
    final r = command['request'] as Map;
    final scope = command['scope'] as String;
    switch (r['action']) {
      case 'list':
        return {
          'installed': registered[scope]!.map(installed).toList(),
          'retained': registered.values
              .expand((v) => v)
              .toSet()
              .map(
                (id) => {
                  'entry': id,
                  'id': id,
                  'name': 'review',
                  'version': id == first ? '1.0' : '2.0',
                  'referenced': false,
                },
              )
              .toList(),
        };
      case 'inspectInstalled':
        if (damaged) {
          throw StateError(
            'Package resource changed. Inspect retained data to export the original ZIP.',
          );
        }
        selected = r['id'] as String;
      case 'inspectRetained':
        selected = r['entry'] as String;
      case 'inspect':
        break;
      case 'cancel':
        return null;
      case 'install':
        if (pending != null) await pending!.future;
        if (failInstall) {
          throw StateError(
            'Could not save catalogue. Retry Install; no active skill changed.',
          );
        }
        registered[scope]!.add(selected);
        return {
          'installed': installed(selected),
          'token': 'token-$tokens',
          'activeSkillChanged': false,
        };
      case 'activate':
        if (stale) {
          throw StateError(
            'Package review expired. Refresh Packages and inspect it again.',
          );
        }
        versions[scope]!.add({
          'version': versions[scope]!.length + 1,
          'document': installed(selected)['document'],
          'package': installed(selected)['package'],
        });
        return active(scope);
      case 'export':
        if (failExport) {
          throw StateError('File already exists. Choose a new filename.');
        }
        return {'bytes': 500};
      case 'remove':
        registered[scope]!.remove(selected);
        return {'filesRemoved': true};
      case 'forgetVersion':
        if (failForget) {
          throw StateError(
            'Could not save. Retry Forget saved version; current version and files remain.',
          );
        }
        if (stale) {
          throw StateError(
            'Saved version changed. Refresh Packages and inspect it again.',
          );
        }
        versions[scope]!.removeWhere((v) => v['version'] == r['version']);
        return {
          'active': active(scope),
          'versionForgotten': r['version'],
          'filesChanged': false,
        };
      default:
        throw StateError('Unexpected command $command');
    }
    return {
      'token': 'token-${++tokens}',
      'package': installed(selected),
      'registered': registered[scope]!.contains(selected),
      'active': active(scope),
      'entry': selected,
    };
  }
}

Future<void> show(
  WidgetTester t,
  PackagesBridge b, {
  bool dark = false,
  bool hasProject = true,
  GlobalKey? key,
}) async {
  await t.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      builder: (_, child) => RepaintBoundary(key: key, child: child!),
      home: Scaffold(
        body: SkillPackages(
          bridge: b,
          session: hasProject ? 'task' : '',
          initialScope: hasProject ? 'project' : 'global',
          hasProject: hasProject,
          onBack: () {},
          chooseArchive: () async => 'C:/fixture/review.zip',
          chooseExport: (_) async => 'C:/fixture/export.zip',
        ),
      ),
    ),
  );
  await t.pumpAndSettle();
}

Future<void> press(WidgetTester t, String key) async {
  final target = find.byKey(Key(key));
  if (target.evaluate().isEmpty) {
    await t.scrollUntilVisible(
      target,
      180,
      scrollable: find
          .descendant(
            of: find.byKey(const Key('packages-scroll')),
            matching: find.byType(Scrollable),
          )
          .first,
    );
  }
  await t.ensureVisible(target);
  if (key == 'confirm-forget-package-version') {
    await t.pump();
  } else {
    await t.pumpAndSettle();
  }
  await t.tap(target);
  if (key == 'remove-package' || key.startsWith('forget-package-version-')) {
    await t.pump();
    await t.pump(const Duration(milliseconds: 300));
  } else {
    await t.pumpAndSettle();
  }
}

Future<void> import(WidgetTester t) async {
  await press(t, 'import-package');
  await t.enterText(find.byKey(const Key('package-name')), 'review');
  await press(t, 'choose-package-zip');
  await press(t, 'review-package-zip');
}

void main() {
  setUpAll(loadReviewFonts);
  testWidgets(
    'forgetting an older activation is confirmed and failed saves preserve the current version',
    (t) async {
      viewport(t, const Size(390, 740));
      for (final dark in [false, true]) {
        final b = PackagesBridge()..failForget = true;
        b.registered['project']!.addAll([first, second]);
        b.versions['project']!.addAll([
          {
            'version': 1,
            'document': installed(first)['document'],
            'package': installed(first)['package'],
          },
          {
            'version': 2,
            'document': installed(second)['document'],
            'package': installed(second)['package'],
          },
        ]);
        final key = GlobalKey();
        await show(t, b, dark: dark, key: key);
        await press(t, 'inspect-package-$second');
        await t.scrollUntilVisible(
          find.text('Saved versions'),
          160,
          scrollable: find
              .descendant(
                of: find.byKey(const Key('packages-scroll')),
                matching: find.byType(Scrollable),
              )
              .first,
        );
        await t.tap(find.text('Saved versions'));
        await t.pumpAndSettle();
        expect(find.byKey(const Key('forget-package-version-2')), findsNothing);
        await press(t, 'package-list');
        await press(t, 'inspect-package-$first');
        await t.scrollUntilVisible(
          find.text('Saved versions'),
          160,
          scrollable: find
              .descendant(
                of: find.byKey(const Key('packages-scroll')),
                matching: find.byType(Scrollable),
              )
              .first,
        );
        await t.tap(find.text('Saved versions'));
        await t.pumpAndSettle();
        await press(t, 'forget-package-version-1');
        expect(find.textContaining('Current v2'), findsOneWidget);
        await capture(
          t,
          key,
          'eh09-package-forget-confirm-${dark ? 'dark' : 'light'}',
        );
        await t.tap(find.text('Keep version'));
        await t.pumpAndSettle();
        expect(b.versions['project']!.length, 2);
        await press(t, 'forget-package-version-1');
        await press(t, 'confirm-forget-package-version');
        expect(find.textContaining('Retry Forget saved version'), findsWidgets);
        expect(b.versions['project']!.length, 2);
        await capture(
          t,
          key,
          'eh09-package-forget-failure-${dark ? 'dark' : 'light'}',
        );
        b.failForget = false;
        await press(t, 'forget-package-version-1');
        await press(t, 'confirm-forget-package-version');
        expect(b.versions['project']!.single['version'], 2);
        expect(b.versions['project']!.single['package']['id'], second);
        expect(b.registered['project'], containsAll([first, second]));
        expect(
          t
              .widget<TextButton>(find.byKey(const Key('remove-package')))
              .onPressed,
          isNotNull,
        );
        expect(find.byKey(const Key('forget-package-version-1')), findsNothing);
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
      }
    },
  );
  testWidgets(
    'selected package context exposes identity and resource base in compact Light and Dark',
    (t) async {
      viewport(t, const Size(390, 740));
      for (final dark in [false, true]) {
        final key = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            builder: (_, child) => RepaintBoundary(key: key, child: child!),
            home: ContextInspector(
              report: {
                'messages': [
                  {
                    'role': 'system',
                    'content': 'Host-pinned reviewed package.',
                  },
                  {'role': 'user', 'content': 'Review the public fixture'},
                ],
                'skills': [
                  {
                    'scope': 'global',
                    'name': 'review',
                    'version': 1,
                    'package': installed(first)['package'],
                  },
                ],
                'skillEntries': [
                  {
                    'scope': 'global',
                    'version': 1,
                    'document': installed(first)['document'],
                  },
                ],
              },
            ),
          ),
        );
        await t.scrollUntilVisible(
          find.text('review · package resources'),
          160,
          scrollable: find.byType(Scrollable).first,
        );
        await t.tap(find.text('review · package resources'));
        await t.pumpAndSettle();
        await t.ensureVisible(find.textContaining('Resource base:'));
        await t.drag(find.byType(ListView).first, const Offset(0, -500));
        await t.pumpAndSettle();
        await capture(
          t,
          key,
          'eh09-resource-context-${dark ? 'dark' : 'light'}',
        );
        expect(find.textContaining('Archive: $first'), findsOneWidget);
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
      }
    },
  );
  testWidgets('Settings Skills exposes packages without a chat or provider', (
    t,
  ) async {
    final b = PackagesBridge();
    await t.pumpWidget(
      MaterialApp(
        home: Scaffold(body: SkillsInspector(bridge: b, hasProject: false)),
      ),
    );
    await t.pumpAndSettle();
    await press(t, 'skill-packages');
    expect(find.byType(SkillPackages), findsOneWidget);
    expect(b.calls.last['scope'], 'global');
    expect(b.calls.last['session'], '');
    await press(t, 'packages-back');
    expect(find.byType(SkillsInspector), findsOneWidget);
    expect(find.byType(SkillPackages), findsNothing);
  });
  for (final scope in ['project', 'global']) {
    testWidgets(
      '$scope install is separate from activation; update and rollback retain identity',
      (t) async {
        final b = PackagesBridge();
        await show(t, b, hasProject: scope == 'project');
        await import(t);
        expect(find.byKey(const Key('activate-package')), findsNothing);
        await press(t, 'install-package');
        expect(b.versions[scope], isEmpty);
        await press(t, 'activate-package');
        expect(b.versions[scope]!.single['package']['id'], first);
        expect(
          t
              .widget<FilledButton>(find.byKey(const Key('activate-package')))
              .onPressed,
          isNull,
        );
        await press(t, 'package-list');
        b.selected = second;
        await import(t);
        await press(t, 'install-package');
        await press(t, 'activate-package');
        expect(b.versions[scope]!.last['package']['id'], second);
        await press(t, 'package-list');
        await press(t, 'inspect-package-$first');
        await t.scrollUntilVisible(
          find.text('Saved versions'),
          160,
          scrollable: find
              .descendant(
                of: find.byKey(const Key('packages-scroll')),
                matching: find.byType(Scrollable),
              )
              .first,
        );
        await t.tap(find.text('Saved versions'));
        await t.pumpAndSettle();
        await press(t, 'rollback-package-1');
        expect(b.calls.where((c) => c['request']?['rollback'] == 1).length, 1);
        expect(b.versions[scope]!.last['package']['id'], first);
        await press(t, 'export-package');
        expect(b.calls.last['request']['action'], 'export');
        expect(t.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'failed installation retries same review; stale activation and export failure preserve work',
    (t) async {
      final b = PackagesBridge()..failInstall = true;
      await show(t, b);
      await import(t);
      await press(t, 'install-package');
      expect(find.textContaining('Retry Install'), findsWidgets);
      expect(b.versions['project'], isEmpty);
      final token = b.calls.last['request']['token'];
      b.failInstall = false;
      await press(t, 'install-package');
      expect(
        b.calls
            .where((c) => c['request']?['action'] == 'install')
            .last['request']['token'],
        token,
      );
      b.stale = true;
      await press(t, 'activate-package');
      expect(b.versions['project'], isEmpty);
      expect(find.textContaining('review expired'), findsWidgets);
      b.failExport = true;
      await press(t, 'export-package');
      expect(find.textContaining('Choose a new filename'), findsWidgets);
      b.failExport = false;
      await press(t, 'export-package');
      expect(find.text('Exported the original ZIP.'), findsOneWidget);
      await press(t, 'refresh-packages');
      expect(b.calls.last['request']['action'], 'list');
    },
  );
  testWidgets(
    'damaged resources keep original export and safe removal recovery reachable',
    (t) async {
      final b = PackagesBridge()..damaged = true;
      b.registered['project']!.add(first);
      await show(t, b);
      await press(t, 'inspect-package-$first');
      expect(find.textContaining('resource changed'), findsWidgets);
      await t.ensureVisible(find.text('Retained data and recovery'));
      await t.tap(find.text('Retained data and recovery'));
      await t.pumpAndSettle();
      await press(t, 'retained-package-$first');
      expect(find.textContaining('Recovery review'), findsOneWidget);
      await press(t, 'export-package');
      expect(b.calls.last['request']['action'], 'export');
      await press(t, 'remove-package');
      await t.tap(find.text('Keep'));
      await t.pumpAndSettle();
      expect(b.registered['project'], contains(first));
      await press(t, 'remove-package');
      await t.tap(find.text('Remove installation'));
      await t.pumpAndSettle();
      expect(b.registered['project'], isEmpty);
    },
  );
  testWidgets(
    'scope changes cancel reviews without activating or losing import inputs',
    (t) async {
      final b = PackagesBridge();
      await show(t, b);
      await import(t);
      await t.tap(find.text('All projects'));
      await t.pumpAndSettle();
      expect(
        b.calls.where((c) => c['request']?['action'] == 'cancel').last['scope'],
        'project',
      );
      expect(b.calls.last['scope'], 'global');
      expect(find.byKey(const Key('install-package')), findsNothing);
      expect(b.versions.values.every((v) => v.isEmpty), isTrue);
      await press(t, 'import-package');
      expect(
        t
            .widget<TextField>(find.byKey(const Key('package-name')))
            .controller!
            .text,
        'review',
      );
      expect(find.text('C:/fixture/review.zip'), findsOneWidget);
    },
  );
  testWidgets(
    'pending install keeps reachable actions disabled and prevents duplicate publication',
    (t) async {
      final b = PackagesBridge()..pending = Completer<void>();
      await show(t, b);
      await import(t);
      await t.tap(find.byKey(const Key('install-package')));
      await t.pump();
      expect(
        t.widget<TextButton>(find.byKey(const Key('packages-back'))).onPressed,
        isNull,
      );
      expect(
        t
            .widget<FilledButton>(find.byKey(const Key('install-package')))
            .onPressed,
        isNull,
      );
      b.pending!.complete();
      await t.pumpAndSettle();
      expect(
        b.calls.where((c) => c['request']?['action'] == 'install').length,
        1,
      );
    },
  );
  testWidgets('compact import update and recovery fit Light and Dark', (
    t,
  ) async {
    viewport(t, const Size(390, 740));
    for (final dark in [false, true]) {
      final b = PackagesBridge();
      final key = GlobalKey();
      final label = dark ? 'dark' : 'light';
      await show(t, b, dark: dark, key: key);
      await press(t, 'import-package');
      await t.enterText(find.byKey(const Key('package-name')), 'review');
      await press(t, 'choose-package-zip');
      await capture(t, key, 'eh09-package-import-$label');
      await press(t, 'review-package-zip');
      await press(t, 'install-package');
      await press(t, 'activate-package');
      await press(t, 'package-list');
      b.selected = second;
      await import(t);
      await capture(t, key, 'eh09-package-update-$label');
      await press(t, 'install-package');
      b.stale = true;
      await press(t, 'activate-package');
      await capture(t, key, 'eh09-package-recovery-$label');
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
    }
  });
}
