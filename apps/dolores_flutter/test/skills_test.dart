import 'dart:async';

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/skills.dart';
import 'package:dolores_flutter/inspector.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class SkillBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool enabled = false, saved = false, missing = false, conflict = false;
  int revision = 1, version = 1;
  Completer<void>? pending;
  Completer<void>? exporting;
  bool exportConflict = false;
  final text =
      '---\nname: review\ndescription: Review code\n---\n@../private.env\n${'Check tests.\n' * 65}';
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'projectSkills':
        return {
          'items': [
            {
              'name': 'review',
              'enabled': enabled,
              'revision': saved ? revision : null,
              'version': saved ? version : null,
              'sourceAvailable': !missing,
            },
          ],
          'partial': false,
          'problem': missing ? 'Source folder unavailable.' : null,
        };
      case 'reviewSkill':
        if (missing && command['version'] == null) {
          throw Exception('SKILL.md is missing.');
        }
        return {
          'token': 'token',
          'document': {
            'name': 'review',
            'description': 'Review code',
            'text': text,
          },
          'enabled': enabled,
          'revision': saved ? revision : null,
          'activeVersion': saved ? version : null,
          'reviewVersion': command['version'],
          'sourceMatches': !missing,
          'alreadyActive': enabled && command['version'] == version,
          'versions': saved
              ? [
                  for (var i = 1; i <= version; i++)
                    {'version': i, 'reviewedAt': 1},
                ]
              : [],
          'problem': missing ? 'SKILL.md is missing.' : null,
        };
      case 'activateSkill':
        if (pending != null) await pending!.future;
        if (conflict) {
          throw Exception('SKILL.md changed after review. Refresh Skills.');
        }
        if (saved) {
          version++;
          revision++;
        } else {
          saved = true;
        }
        enabled = true;
        return {
          'name': 'review',
          'versions': [
            {'version': version},
          ],
        };
      case 'disableSkill':
        enabled = false;
        revision++;
        return null;
      case 'exportSkill':
        if (exporting != null) await exporting!.future;
        if (exportConflict) {
          throw Exception('That file already exists. Choose a new folder.');
        }
        return {'name': 'review', 'version': 1};
      case 'forgetSkill':
        saved = false;
        enabled = false;
        return null;
      default:
        return null;
    }
  }
}

class ScopedSkillBridge implements ChatBridge {
  final project = SkillBridge(), global = SkillBridge();
  final commands = <Map<String, dynamic>>[];
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    final scope = command['scope'] as String? ?? 'project';
    final result = await (scope == 'global' ? global : project).call(command);
    if (result is Map) return {...result, 'scope': scope};
    return result;
  }
}

Future<void> open(
  WidgetTester tester,
  ChatBridge bridge,
  bool dark, {
  bool hasProject = true,
  Future<String?> Function(String name, int version)? chooseExportPath,
}) async {
  tester.view.physicalSize = const Size(390, 700);
  tester.view.devicePixelRatio = 1;
  addTearDown(tester.view.resetPhysicalSize);
  addTearDown(tester.view.resetDevicePixelRatio);
  await tester.pumpWidget(
    MaterialApp(
      theme: doloresTheme(dark),
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showDialog<void>(
              context: context,
              builder: (_) => SkillsInspector(
                bridge: bridge,
                session: 'chat',
                hasProject: hasProject,
                chooseExportPath: chooseExportPath,
              ),
            ),
            child: const Text('Open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('Open'));
  await tester.pumpAndSettle();
}

Future<void> press(WidgetTester tester, String key) async {
  final target = find.byKey(Key(key));
  if (key.startsWith('review-skill') ||
      key.startsWith('saved-skill') ||
      key.startsWith('skill-version')) {
    await tester.scrollUntilVisible(
      target,
      150,
      scrollable: find.byType(Scrollable).first,
    );
    await Scrollable.ensureVisible(tester.element(target), alignment: .5);
    await tester.pumpAndSettle();
  }
  await tester.tap(target);
  await tester.pumpAndSettle();
}

void main() {
  for (final dark in [false, true]) {
    testWidgets(
      'export selected retained version with no activation in compact ${dark ? 'dark' : 'light'} layout',
      (tester) async {
        final bridge = SkillBridge()
          ..saved = true
          ..version = 2
          ..missing = true;
        final selections = <Object>[];
        await open(
          tester,
          bridge,
          dark,
          chooseExportPath: (name, version) async {
            selections.addAll([name, version]);
            return '/chosen/review/SKILL.md';
          },
        );
        await press(tester, 'saved-skill-review');
        await press(tester, 'skill-version-1');
        await press(tester, 'export-skill');
        expect(selections, ['review', 1]);
        expect(
          bridge.commands.where((c) => c['command'] == 'exportSkill').single,
          {
            'command': 'exportSkill',
            'session': 'chat',
            'scope': 'project',
            'token': 'token',
            'path': '/chosen/review/SKILL.md',
          },
        );
        expect(
          find.textContaining('Exported review version 1'),
          findsOneWidget,
        );
        expect(bridge.enabled, isFalse);
        expect(bridge.version, 2);
        expect(
          bridge.commands.where((c) => c['command'] == 'activateSkill'),
          isEmpty,
        );
        expect(tester.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'export absent for unretained file and cancelling destination publishes nothing',
    (tester) async {
      final bridge = SkillBridge()..saved = true;
      await open(tester, bridge, true, chooseExportPath: (_, _) async => null);
      await press(tester, 'review-skill-review');
      expect(find.byKey(const Key('export-skill')), findsNothing);
      await press(tester, 'skills-back');
      await press(tester, 'saved-skill-review');
      await press(tester, 'export-skill');
      expect(
        bridge.commands.where((c) => c['command'] == 'exportSkill'),
        isEmpty,
      );
      expect(bridge.enabled, isFalse);
    },
  );
  testWidgets(
    'side-chat export stays global; pending and failed export retain the review for retry',
    (tester) async {
      final bridge = ScopedSkillBridge();
      bridge.global
        ..saved = true
        ..exportConflict = true
        ..exporting = Completer<void>();
      await open(
        tester,
        bridge,
        false,
        hasProject: false,
        chooseExportPath: (_, _) async => '/chosen/review/SKILL.md',
      );
      await press(tester, 'saved-skill-review');
      await tester.tap(find.byKey(const Key('export-skill')));
      await tester.pump();
      expect(
        tester
            .widget<TextButton>(find.byKey(const Key('export-skill')))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      bridge.global.exporting!.complete();
      await tester.pumpAndSettle();
      expect(find.textContaining('That file already exists'), findsOneWidget);
      expect(
        tester
            .widget<TextButton>(find.byKey(const Key('export-skill')))
            .onPressed,
        isNotNull,
      );
      bridge.global.exportConflict = false;
      await press(tester, 'export-skill');
      final calls = bridge.commands
          .where((c) => c['command'] == 'exportSkill')
          .toList();
      expect(calls.length, 2);
      expect(
        calls.every((c) => c['scope'] == 'global' && c['token'] == 'token'),
        isTrue,
      );
      expect(bridge.global.enabled, isFalse);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets(
    'side-chat Skills action opens without a working folder and locks chat changes',
    (tester) async {
      final bridge = ScopedSkillBridge();
      final chat = ChatController(bridge)
        ..session = 'chat'
        ..workspaceKind = 'side'
        ..loading = false;
      addTearDown(chat.dispose);
      await tester.pumpWidget(
        MaterialApp(
          theme: doloresTheme(true),
          home: Scaffold(
            body: Builder(
              builder: (context) => TextButton(
                onPressed: () => showSkills(context, chat),
                child: const Text('Open'),
              ),
            ),
          ),
        ),
      );
      await tester.tap(find.text('Open'));
      await tester.pumpAndSettle();
      expect(chat.changing, isTrue);
      expect(find.byType(SkillsInspector), findsOneWidget);
      expect(
        tester
            .widget<SegmentedButton<String>>(
              find.byType(SegmentedButton<String>),
            )
            .selected,
        {'global'},
      );
      chat.newChat();
      expect(chat.session, 'chat');
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(chat.changing, isFalse);
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets('global and project tabs keep activation and disable scoped', (
    tester,
  ) async {
    final bridge = ScopedSkillBridge();
    await open(tester, bridge, true);
    await tester.tap(find.text('All projects'));
    await tester.pumpAndSettle();
    await press(tester, 'review-skill-review');
    await press(tester, 'activate-skill');
    expect(bridge.global.enabled, isTrue);
    expect(bridge.project.enabled, isFalse);
    await tester.tap(find.text('This project'));
    await tester.pumpAndSettle();
    await press(tester, 'review-skill-review');
    await press(tester, 'activate-skill');
    expect(bridge.project.enabled, isTrue);
    await press(tester, 'disable-skill');
    expect(bridge.project.enabled, isFalse);
    expect(bridge.global.enabled, isTrue);
    expect(
      bridge.commands
          .where((c) => c['command'] == 'disableSkill')
          .single['scope'],
      'project',
    );
    expect(tester.takeException(), isNull);
  });
  testWidgets('side chats open global skills with project selection disabled', (
    tester,
  ) async {
    final bridge = ScopedSkillBridge();
    await open(tester, bridge, false, hasProject: false);
    final selector = tester.widget<SegmentedButton<String>>(
      find.byType(SegmentedButton<String>),
    );
    expect(selector.selected, {'global'});
    expect(selector.segments.first.enabled, isFalse);
    await press(tester, 'review-skill-review');
    await press(tester, 'activate-skill');
    expect(bridge.global.enabled, isTrue);
    expect(bridge.commands.every((c) => c['scope'] == 'global'), isTrue);
    expect(tester.takeException(), isNull);
  });
  for (final dark in [false, true]) {
    testWidgets(
      'compact literal review, explicit activation and missing-source rollback ${dark ? 'dark' : 'light'}',
      (tester) async {
        final bridge = SkillBridge();
        await open(tester, bridge, dark);
        await press(tester, 'review-skill-review');
        expect(bridge.enabled, isFalse);
        await tester.scrollUntilVisible(
          find.byKey(const Key('skill-source-text')),
          150,
          scrollable: find.byType(Scrollable).first,
        );
        expect(
          tester
              .widget<SelectableText>(
                find.byKey(const Key('skill-source-text')),
              )
              .data,
          contains('@../private.env'),
        );
        await press(tester, 'activate-skill');
        expect(bridge.enabled, isTrue);
        await press(tester, 'skills-back');
        bridge.missing = true;
        await press(tester, 'saved-skill-review');
        await tester.scrollUntilVisible(
          find.textContaining('file is unavailable'),
          150,
          scrollable: find.byType(Scrollable).first,
        );
        expect(find.textContaining('file is unavailable'), findsOneWidget);
        await press(tester, 'disable-skill');
        expect(bridge.enabled, isFalse);
        await press(tester, 'saved-skill-review');
        await press(tester, 'skill-version-1');
        await press(tester, 'activate-skill');
        expect(bridge.version, 2);
        expect(bridge.enabled, isTrue);
        await press(tester, 'forget-skill');
        expect(bridge.saved, isFalse);
        expect(tester.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'pending write excludes close and repeated activation; failed review cannot be replayed',
    (tester) async {
      final bridge = SkillBridge()
        ..pending = Completer<void>()
        ..conflict = true;
      await open(tester, bridge, true);
      await press(tester, 'review-skill-review');
      await tester.tap(find.byKey(const Key('activate-skill')));
      await tester.pump();
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('activate-skill')))
            .onPressed,
        isNull,
      );
      bridge.pending!.complete();
      await tester.pumpAndSettle();
      expect(bridge.enabled, isFalse);
      await tester.scrollUntilVisible(
        find.textContaining('changed after review'),
        150,
        scrollable: find.byType(Scrollable).first,
      );
      expect(find.textContaining('changed after review'), findsOneWidget);
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('activate-skill')))
            .onPressed,
        isNull,
      );
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(
        bridge.commands.where((c) => c['command'] == 'activateSkill').length,
        1,
      );
      expect(tester.takeException(), isNull);
    },
  );
  testWidgets('Close cancels a ready review without activation', (
    tester,
  ) async {
    final bridge = SkillBridge();
    await open(tester, bridge, false);
    await press(tester, 'review-skill-review');
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();
    expect(bridge.commands.last['command'], 'cancelSkillReview');
    expect(bridge.enabled, isFalse);
  });
  testWidgets('context shows exact active snapshot and version', (
    tester,
  ) async {
    await tester.pumpWidget(
      MaterialApp(
        theme: doloresTheme(true),
        home: ContextInspector(
          report: {
            'messages': [
              {'role': 'system', 'content': 'rules'},
              {'role': 'user', 'content': ''},
            ],
            'skills': [
              {'name': 'review', 'version': 3},
            ],
            'skillEntries': [
              {
                'version': 3,
                'document': {'name': 'review', 'text': 'Exact reviewed body'},
              },
            ],
          },
        ),
      ),
    );
    await tester.scrollUntilVisible(
      find.text('Skills · reviewed snapshots'),
      200,
      scrollable: find.byType(Scrollable).first,
    );
    await tester.tap(find.text('Skills · reviewed snapshots'));
    await tester.pumpAndSettle();
    expect(find.text('Exact reviewed body'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
}
