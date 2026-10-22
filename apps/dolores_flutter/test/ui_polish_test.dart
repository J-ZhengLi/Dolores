import 'support/dock_navigation.dart';

import 'package:flutter/gestures.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:dolores_flutter/workspace_dock.dart';
import 'package:dolores_flutter/form_field_layout.dart';
import 'package:dolores_flutter/help_tip.dart';
import 'package:dolores_flutter/models.dart';
import 'package:dolores_flutter/page_transition.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/usage_details.dart';

import 'models_test.dart' show DetailsBridge;
import 'frost_design_test.dart' show capture, viewport, DesignBridge;
import 'support/render_fonts.dart';

void main() {
  for (final mode in [ThemeMode.light, ThemeMode.dark]) {
    testWidgets(
      'floating dock gives Source Control full height and preserves drafts ($mode)',
      (t) async {
        viewport(t, const Size(1200, 800));
        await t.runAsync(loadReviewFonts);
        final chat = ChatController(DesignBridge())
          ..loading = false
          ..configured = true
          ..session = 'public-fixture'
          ..workspaceRoot = 'C:/fixture/notes'
          ..workspaceKind = 'project'
          ..draft = 'Keep my chat draft';
        final host = AppHost(chat);
        final repository = GitWorkspace(chat.workspaceRoot!, chat.session!)
          ..status = {
            'root': chat.workspaceRoot,
            'branch': 'main',
            'head': null,
            'entries': [
              {
                'path': 'README.md',
                'index': ' ',
                'worktree': 'M',
                'conflict': false,
              },
              {
                'path': 'lib/notes.dart',
                'index': 'M',
                'worktree': ' ',
                'conflict': false,
              },
            ],
          };
        host.git.workspaces[repository.root] = repository;
        final key = GlobalKey();
        await t.pumpWidget(
          DoloresApp(chat: chat, host: host, themeMode: mode, captureKey: key),
        );
        await t.pumpAndSettle();
        await tapDockItem(t, 'page-sourceControl');
        await t.pumpAndSettle();
        final sidebar = t.getRect(find.byType(SourceControlPanel));
        final dock = t.getRect(find.byKey(const Key('workspace-dock')));
        expect(sidebar.top, lessThan(70));
        expect(sidebar.height, greaterThan(650));
        expect(sidebar.bottom, greaterThan(dock.top));
        expect(sidebar.bottom, closeTo(800, 1));
        await t.enterText(
          find.byKey(const Key('git-commit-message')),
          'Keep my commit draft',
        );
        await t.pump();
        await capture(t, key, 'dock-source-control-${mode.name}');
        expect(find.text('Dolores'), findsOneWidget);
        expect(
          t.getRect(find.byKey(const Key('title-panel-toggle'))).left,
          greaterThan(t.getRect(find.text('Dolores')).right),
        );
        await t.pump(const Duration(seconds: 3));
        await t.pumpAndSettle();
        expect(find.byKey(const Key('dock-reveal')), findsOneWidget);
        expect(find.byKey(const Key('page-home')).hitTestable(), findsNothing);
        expect(t.getRect(find.byType(SourceControlPanel)), sidebar);
        await capture(t, key, 'dock-hidden-${mode.name}');
        final pointer = await t.createGesture(kind: PointerDeviceKind.mouse);
        await pointer.addPointer(location: const Offset(1000, 300));
        await pointer.moveTo(t.getCenter(find.byKey(const Key('dock-reveal'))));
        await t.pumpAndSettle();
        await pointer.moveTo(
          t.getCenter(find.byKey(const Key('page-sourceControl'))),
        );
        await t.pump(const Duration(seconds: 3));
        expect(find.byKey(const Key('dock-reveal')), findsNothing);
        await pointer.moveTo(const Offset(1000, 300));
        await t.pump(const Duration(milliseconds: 700));
        await t.pumpAndSettle();
        expect(find.byKey(const Key('dock-reveal')), findsOneWidget);
        await pointer.removePointer();
        await revealDock(t);
        await t.pump(const Duration(seconds: 3));
        expect(
          t
              .state<WorkspaceDockState>(find.byType(WorkspaceDock))
              .focus
              .hasFocus,
          isTrue,
        );
        expect(find.byKey(const Key('dock-reveal')), findsNothing);
        await tapDockItem(t, 'page-home');
        await t.pumpAndSettle();
        expect(chat.draft, 'Keep my chat draft');
        await tapDockItem(t, 'page-sourceControl');
        await t.pumpAndSettle();
        expect(find.text('Keep my commit draft'), findsOneWidget);
        host.togglePanel();
        t.view.physicalSize = const Size(420, 640);
        await t.pumpAndSettle();
        final compactDock = t.getRect(find.byKey(const Key('workspace-dock')));
        expect(compactDock.left, greaterThanOrEqualTo(0));
        expect(compactDock.right, lessThanOrEqualTo(420));
        await t.tap(find.byKey(const Key('title-panel-toggle')));
        await t.pumpAndSettle();
        expect(find.text('Keep my commit draft'), findsOneWidget);
        await capture(t, key, 'dock-compact-${mode.name}');
        await t.tapAt(const Offset(410, 200));
        await t.pumpAndSettle();
        expect(host.panelHidden, isTrue);
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        host.dispose();
      },
    );
  }

  testWidgets(
    'page travel keeps drafts and retargets rapid input; reduced motion is immediate',
    (t) async {
      var selected = 'one';
      var reduced = false;
      late StateSetter update;
      final input = TextEditingController(text: 'Keep this draft');
      addTearDown(input.dispose);
      await t.pumpWidget(
        MaterialApp(
          home: StatefulBuilder(
            builder: (context, setState) {
              update = setState;
              return MediaQuery(
                data: MediaQuery.of(context)
                    .copyWith(disableAnimations: reduced),
                child: Scaffold(
                  body: PageTransitionStack(
                    selected: selected,
                    pages: {
                      'one': TextField(controller: input),
                      'two': const Center(child: Text('Second page')),
                      'three': const Center(child: Text('Third page')),
                    },
                  ),
                ),
              );
            },
          ),
        ),
      );
      update(() => selected = 'two');
      await t.pump();
      await t.pump(const Duration(milliseconds: 60));
      final opacity = t.widget<Opacity>(
        find.ancestor(
          of: find.text('Second page'),
          matching: find.byType(Opacity),
        ),
      );
      expect(opacity.opacity, allOf(greaterThan(0), lessThan(1)));
      update(() => selected = 'three');
      await t.pumpAndSettle();
      expect(find.text('Second page'), findsNothing);
      expect(find.text('Third page'), findsOneWidget);
      update(() {
        reduced = true;
        selected = 'one';
      });
      await t.pump();
      expect(find.text('Keep this draft'), findsOneWidget);
      expect(
        t
            .widget<Opacity>(
              find.ancestor(
                of: find.byType(TextField),
                matching: find.byType(Opacity),
              ),
            )
            .opacity,
        1,
      );
      expect(t.binding.transientCallbackCount, 0);
    },
  );

  testWidgets('long field help floats on hover, keyboard focus and tap', (
    t,
  ) async {
    const explanation =
        'This detailed explanation stays out of the form until requested. The saved value and validation stay visible.';
    await t.pumpWidget(
      MaterialApp(
        theme: doloresTheme(false),
        home: const Scaffold(
          body: Padding(
            padding: EdgeInsets.all(20),
            child: FieldLabel(
              label: 'A setting',
              help: explanation,
              child: TextField(),
            ),
          ),
        ),
      ),
    );
    expect(find.text(explanation), findsNothing);
    final help = find.byType(HelpTip);
    final pointer = await t.createGesture(kind: PointerDeviceKind.mouse);
    await pointer.addPointer(location: Offset.zero);
    await pointer.moveTo(t.getCenter(help));
    await t.pump(const Duration(milliseconds: 400));
    await t.pumpAndSettle();
    expect(find.text(explanation), findsOneWidget);
    await pointer.moveTo(Offset.zero);
    await pointer.removePointer();
    await t.pumpAndSettle();
    expect(find.text(explanation), findsNothing);
    await t.sendKeyEvent(LogicalKeyboardKey.tab);
    await t.pumpAndSettle();
    expect(find.text(explanation), findsOneWidget);
    await t.sendKeyEvent(LogicalKeyboardKey.tab);
    await t.pumpAndSettle();
    expect(find.text(explanation), findsNothing);
    await t.tap(help);
    await t.pumpAndSettle();
    expect(find.text(explanation), findsOneWidget);
  });

  for (final dark in [false, true]) {
    testWidgets(
      'aligned model form and concise run details render in both widths ($dark)',
      (t) async {
        viewport(t, const Size(1000, 800));
        await t.runAsync(loadReviewFonts);
        final chat = ChatController(DetailsBridge())..loading = false;
        final key = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            builder: (_, child) => RepaintBoundary(key: key, child: child!),
            home: ModelDetailsEditor(
              chat: chat,
              model: 'deepseek-v4.1-flash-1017',
            ),
          ),
        );
        await t.pumpAndSettle();
        final field = find.byKey(const Key('model-detail-context'));
        final label = find.text('Context window (tokens)');
        expect(t.getBottomLeft(label).dy, lessThan(t.getTopLeft(field).dy));
        expect(t.getTopLeft(label).dx, t.getTopLeft(field).dx);
        expect(t.widget<TextField>(field).decoration?.labelText, isNull);
        await capture(t, key, 'model-form-${dark ? 'dark' : 'light'}');
        t.view.physicalSize = const Size(420, 640);
        await t.pumpAndSettle();
        await t.enterText(field, '1000000');
        await t.tap(find.byKey(const Key('model-detail-advanced')));
        await t.pumpAndSettle();
        await capture(t, key, 'model-form-compact-${dark ? 'dark' : 'light'}');
        expect(t.takeException(), isNull);
        await t.pumpWidget(const SizedBox());
        chat.dispose();

        const run = <String, dynamic>{
          'model': 'deepseek-v4.1-flash-1017',
          'agent': {
            'modelCalls': 1,
            'usageByCall': [
              {
                'inputTokens': 3312,
                'outputTokens': 304,
                'totalTokens': 3616,
                'cachedInputTokens': 3072,
                'reasoningTokens': 153,
              },
            ],
          },
          'requestSettings': {'maxOutputTokens': null, 'timeoutSeconds': 180},
          'context': {
            'includedTurns': 6,
            'savedTurns': 6,
            'omittedTurns': 0,
            'tokens': {
              'inputTokens': 7484,
              'contextWindowTokens': 1000000,
              'reservedOutputTokens': 32768,
              'maxInputTokens': 917232,
            },
            'memory': {'used': [], 'omitted': 1},
          },
        };
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            builder: (_, child) => RepaintBoundary(key: key, child: child!),
            home: const Scaffold(body: UsageDetails(metadata: run)),
          ),
        );
        await t.tap(find.text('Run details · 1 model calls'));
        await t.pumpAndSettle();
        expect(find.text('3,312'), findsOneWidget);
        expect(find.text('3,616'), findsOneWidget);
        expect(find.text('Cached input'), findsNothing);
        expect(find.text('Provider default'), findsNothing);
        await capture(t, key, 'run-compact-${dark ? 'dark' : 'light'}');
        t.view.physicalSize = const Size(1000, 800);
        await t.pumpAndSettle();
        await capture(t, key, 'run-${dark ? 'dark' : 'light'}');
        await t.tap(find.byKey(const Key('token-breakdown')));
        await t.pumpAndSettle();
        expect(find.text('3,072'), findsOneWidget);
        expect(find.text('153'), findsOneWidget);
        expect(t.takeException(), isNull);
      },
    );
  }
}
