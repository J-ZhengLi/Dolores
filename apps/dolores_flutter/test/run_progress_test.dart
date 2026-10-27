import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/run_progress.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/workspace_shell.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'app_host_test.dart' show HostBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

void main() {
  testWidgets('elapsed task time ticks locally and queue wording is explicit', (
    t,
  ) async {
    var elapsed = const Duration(seconds: 52);
    await t.pumpWidget(MaterialApp(home: RunElapsed(elapsed: () => elapsed)));
    expect(find.text('Working for 52s'), findsOneWidget);
    elapsed = const Duration(seconds: 65);
    await t.pump(const Duration(seconds: 1));
    expect(find.text('Working for 1m 5s'), findsOneWidget);
    await t.pumpWidget(
      MaterialApp(home: RunElapsed(elapsed: () => elapsed, queued: true)),
    );
    expect(find.text('Queued for 1m 5s'), findsOneWidget);
    await t.pumpWidget(const SizedBox.shrink());
    expect(t.binding.transientCallbackCount, 0);
  });
  testWidgets(
    'status survives rebuilds then rotates without repeating itself',
    (t) async {
      Widget view() => MaterialApp(home: RunStatus(phase: 'reasoning'));
      await t.pumpWidget(view());
      String label() => t
          .widget<Text>(
            find.descendant(
              of: find.byKey(const Key('run-status')),
              matching: find.byType(Text),
            ),
          )
          .data!;
      final first = label();
      for (var i = 0; i < 3; i++) {
        await t.pumpWidget(view());
        await t.pump(const Duration(seconds: 1));
        expect(label(), first);
      }
      await t.pump(const Duration(seconds: 5));
      expect(label(), isNot(first));
      await t.pumpWidget(const SizedBox.shrink());
      expect(t.binding.transientCallbackCount, 0);
    },
  );
  testWidgets(
    'review, questions, queue and Stop are stable; reduced motion is still',
    (t) async {
      for (final entry in {
        'review': 'Waiting for your review',
        'question': 'Waiting for your answer',
        'queued': 'Waiting for a task slot',
        'stopping': 'Stopping…',
      }.entries) {
        await t.pumpWidget(MaterialApp(home: RunStatus(phase: entry.key)));
        await t.pumpAndSettle();
        expect(find.text(entry.value), findsOneWidget);
      }
      await t.pumpWidget(
        MaterialApp(
          builder: (context, child) => MediaQuery(
            data: MediaQuery.of(context).copyWith(disableAnimations: true),
            child: child!,
          ),
          home: const RunStatus(phase: 'reasoning'),
        ),
      );
      await t.pumpAndSettle();
      final label = t.widget<Text>(find.byType(Text)).data;
      await t.pump(const Duration(seconds: 16));
      expect(t.widget<Text>(find.byType(Text)).data, label);
      expect(t.binding.transientCallbackCount, 0);
      await t.pumpWidget(const SizedBox.shrink());
    },
  );
  for (final dark in [false, true]) {
    for (final width in [420.0, 1000.0]) {
      testWidgets('inline running chat dark=$dark width=$width', (t) async {
        viewport(t, Size(width, 800));
        await t.runAsync(loadReviewFonts);
        final chat = ChatController(HostBridge())
          ..loading = false
          ..configured = true
          ..model = 'fixture'
          ..busy = true
          ..pendingInput = 'Inspect the project and verify the result.'
          ..partial = 'I’ll trace the implementation, then verify the behavior.'
          ..modelActive = true
          ..modelPhase = 'reasoning';
        final host = AppHost(chat);
        host.panelHidden = width < 800;
        addTearDown(host.dispose);
        final key = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: RepaintBoundary(
              key: key,
              child: WorkspaceShell(host: host),
            ),
          ),
        );
        await t.pump(const Duration(milliseconds: 500));
        await t.pump(const Duration(milliseconds: 500));
        final timer = find.byKey(const Key('run-elapsed'));
        final status = find.byKey(const Key('run-status'));
        expect(timer, findsOneWidget);
        expect(status, findsOneWidget);
        expect(t.getRect(timer).bottom, lessThan(t.getRect(status).top));
        expect(find.byTooltip('Stop response'), findsOneWidget);
        expect(find.byKey(const Key('running-tasks-menu')), findsOneWidget);
        expect(find.textContaining('· Running'), findsNothing);
        expect(t.takeException(), isNull);
        await capture(
          t,
          key,
          'run-progress-${dark ? 'dark' : 'light'}-${width.toInt()}',
        );
        await t.pumpWidget(const SizedBox.shrink());
      });
    }
  }
}
