import 'dart:async';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/skills.dart';
import 'package:dolores_flutter/inspector.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

class SkillBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool enabled = true, saved = false, missing = false, conflict = false;
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
      case 'setSkillEnabled':
        if (conflict) {
          throw Exception('Could not save this setting. Try again.');
        }
        enabled = command['enabled'] == true;
        saved = true;
        return null;
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

class LongSkillBridge extends SkillBridge {
  @override
  String get text =>
      '---\nname: review\ndescription: Review code\n---\n${'Connection checks 世界\n' * 1200}FULL_SOURCE_END';
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
              builder: (_) => RepaintBoundary(
                key: const Key('skills-capture'),
                child: SkillsInspector(
                  bridge: bridge,
                  session: 'chat',
                  hasProject: hasProject,
                  chooseExportPath: chooseExportPath,
                ),
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
      key.startsWith('toggle-skill') ||
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
      'View file retains complete long skill in compact settings $dark',
      (tester) async {
        final bridge = LongSkillBridge();
        await open(tester, bridge, dark);
        await press(tester, 'review-skill-review');
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
          bridge.text,
        );
        expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
        expect(tester.takeException(), isNull);
      },
    );
  }
  setUpAll(() async {
    for (final font in {
      'Segoe UI':
          '${Platform.environment['WINDIR'] ?? 'C:/Windows'}/Fonts/segoeui.ttf',
      'MaterialIcons': '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
    }.entries) {
      final file = File(font.value);
      if (await file.exists()) {
        final loader = FontLoader(font.key)
          ..addFont(
            Future.value(ByteData.sublistView(await file.readAsBytes())),
          );
        await loader.load();
      }
    }
  });
  for (final dark in [false, true]) {
    testWidgets(
      'export selected retained version with no activation in compact ${dark ? 'dark' : 'light'} layout',
      (tester) async {
        final bridge = SkillBridge()
          ..enabled = false
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
      final bridge = SkillBridge()
        ..enabled = false
        ..saved = true;
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
        ..enabled = false
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
  for (final dark in [false, true]) {
    testWidgets(
      'files are ready without review, details optional, toggle scoped $dark',
      (tester) async {
        final bridge = ScopedSkillBridge();
        await open(tester, bridge, dark);
        expect(find.text('Ready to use'), findsOneWidget);
        expect(find.text('Review file'), findsNothing);
        await tester.runAsync(() async {
          final boundary = tester.renderObject<RenderRepaintBoundary>(
            find.byKey(const Key('skills-capture')),
          );
          final image = await boundary.toImage(pixelRatio: 1);
          final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
          final directory = Directory('../../output/direct-skills');
          await directory.create(recursive: true);
          await File('${directory.path}/skills-${dark ? 'dark' : 'light'}.png')
              .writeAsBytes(bytes!.buffer.asUint8List());
          image.dispose();
        });
        expect(
          bridge.commands.where((c) => c['command'] == 'reviewSkill'),
          isEmpty,
        );
        await press(tester, 'toggle-skill-review');
        expect(bridge.project.enabled, isFalse);
        expect(bridge.global.enabled, isTrue);
        await tester.tap(find.text('All projects'));
        await tester.pumpAndSettle();
        expect(find.text('Ready to use'), findsOneWidget);
        await press(tester, 'review-skill-review');
        expect(find.byKey(const Key('activate-skill')), findsNothing);
        await tester.scrollUntilVisible(
          find.byKey(const Key('skill-source-text')),
          150,
          scrollable: find.byType(Scrollable).first,
        );
        expect(find.byKey(const Key('skill-source-text')), findsOneWidget);
        expect(
          bridge.commands.where((c) => c['command'] == 'activateSkill'),
          isEmpty,
        );
        expect(tester.takeException(), isNull);
      },
    );
  }
  testWidgets('failed disable preserves ready skill and allows retry', (
    tester,
  ) async {
    final bridge = SkillBridge()..conflict = true;
    await open(tester, bridge, false);
    await press(tester, 'toggle-skill-review');
    expect(bridge.enabled, isTrue);
    expect(find.textContaining('Could not save this setting'), findsOneWidget);
    bridge.conflict = false;
    await press(tester, 'toggle-skill-review');
    expect(bridge.enabled, isFalse);
    expect(find.text('Skill disabled.'), findsOneWidget);
    expect(tester.takeException(), isNull);
  });
  testWidgets('Close cancels a ready review without activation', (
    tester,
  ) async {
    final bridge = SkillBridge();
    await open(tester, bridge, false);
    await press(tester, 'review-skill-review');
    await tester.tap(find.text('Close'));
    await tester.pumpAndSettle();
    expect(bridge.commands.last['command'], 'cancelSkillReview');
    expect(bridge.enabled, isTrue);
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
