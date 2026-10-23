import 'dart:convert';
import 'dart:io';
import 'dart:ui' as ui;

import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/main.dart';
import 'package:dolores_flutter/worktrees.dart';
import 'package:dolores_flutter/worktree_transfer.dart';
import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';

import 'support/render_fonts.dart';

void main() {
  testWidgets(
    'native worktree review, stale refusal, creation and draft-safe navigation',
    (t) async {
      final data = Directory(Platform.environment['DOLORES_DATA_DIR']!);
      final output = Directory('../../output').absolute
          .resolveSymbolicLinksSync();
      expect(
        data.parent.resolveSymbolicLinksSync(),
        startsWith('$output${Platform.pathSeparator}'),
      );
      final project = Directory('${data.parent.path}/worktree-public-project')
        ..createSync();
      final noHooks = Directory('${data.parent.path}/empty-hooks')
        ..createSync();
      Future<String> git(List<String> args) async {
        final result = (await t.runAsync(
          () => Process.run('git', [
            '-c',
            'core.hooksPath=${noHooks.path}',
            '-c',
            'commit.gpgSign=false',
            '-C',
            project.path,
            ...args,
          ]),
        ))!;
        expect(result.exitCode, 0, reason: '${result.stderr}');
        return '${result.stdout}'.trim();
      }

      await git(['init', '-b', 'main']);
      await git(['config', 'user.name', 'Public fixture']);
      await git(['config', 'user.email', 'fixture@example.invalid']);
      await git(['config', 'core.autocrlf', 'false']);
      File('${project.path}/note.txt').writeAsStringSync('base\n');
      await git(['add', 'note.txt']);
      await git(['commit', '-m', 'Public fixture']);
      final commit = await git(['rev-parse', 'HEAD']);
      File('${project.path}/note.txt')
          .writeAsStringSync('keep original dirty file\n');
      final index = File('${project.path}/.git/index').readAsBytesSync();
      final bridge = (await t.runAsync(() async {
        final bridge = NativeBridge();
        await bridge.open(
          testLibrary: Platform.environment['DOLORES_AUDIT_LIBRARY'],
        );
        await loadReviewFonts();
        return bridge;
      }))!;
      final chat = ChatController(bridge, ownsBridge: false)..loading = false;
      addTearDown(() async {
        chat.dispose();
        await bridge.close();
      });
      Future<dynamic> call(Map<String, dynamic> command) async {
        final envelope = (await t.runAsync(() async {
          try {
            return {'value': await bridge.call(command)};
          } catch (error) {
            return {'error': '$error'};
          }
        }))!;
        if (envelope.containsKey('error')) {
          throw StateError('${envelope['error']}');
        }
        return envelope['value'];
      }

      Future<void> waitUntil(bool Function() ready) async {
        final deadline = DateTime.now().add(const Duration(seconds: 40));
        while (!ready() && DateTime.now().isBefore(deadline)) {
          await t.pump(const Duration(milliseconds: 50));
          await t.runAsync(
            () => Future<void>.delayed(const Duration(milliseconds: 20)),
          );
        }
        expect(
          ready(),
          true,
          reason: 'Native worktree flow did not finish; ${chat.error}',
        );
      }

      Future<void> tap(String label) async {
        await waitUntil(() {
          if (find.text(label).evaluate().isEmpty) return false;
          final buttons = find.ancestor(
            of: find.text(label).last,
            matching: find.byWidgetPredicate((w) => w is ButtonStyleButton),
          );
          return buttons.evaluate().any(
            (e) => (e.widget as ButtonStyleButton).onPressed != null,
          );
        });
        await t.ensureVisible(find.text(label).last);
        await t.pump();
        await t.tap(find.text(label).last);
        await t.pump();
      }

      await t.runAsync(chat.refresh);
      final source =
          (await call({
                'command': 'createSession',
                'kind': 'project',
                'path': project.path,
              }))['session']['id']
              as String;
      await t.runAsync(() => chat.select(source));
      chat.draft = 'Keep my source draft 世界';
      t.view.physicalSize = const Size(1100, 900);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final key = GlobalKey();
      await t.pumpWidget(
        RepaintBoundary(
          key: key,
          child: DoloresApp(chat: chat),
        ),
      );
      await t.pumpAndSettle();
      await t.tap(find.byKey(const Key('export-chat')));
      await t.pumpAndSettle();
      await t.tap(find.text('Worktrees…'));
      await waitUntil(
        () =>
            find.text('Review creation').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      await t.enterText(find.byType(TextField).last, 'main');
      await t.pump();
      await tap('Review creation');
      await waitUntil(() => find.text('Create worktree').evaluate().isNotEmpty);
      expect(find.text(commit), findsOneWidget);
      // Stale reviews must refuse before creating a folder or losing the draft.
      await git(['config', 'fixture.changed', 'after-review']);
      await tap('Create worktree');
      await waitUntil(
        () => find
            .textContaining('Repository configuration changed')
            .evaluate()
            .isNotEmpty,
      );
      expect(chat.session, source);
      expect(chat.draft, 'Keep my source draft 世界');
      var retained = await call({
        'command': 'worktrees',
        'session': source,
        'request': {'action': 'list'},
      });
      expect(retained['items'], isEmpty);
      await tap('Review creation');
      await waitUntil(() => find.text('Create worktree').evaluate().isNotEmpty);
      await tap('Create worktree');
      await waitUntil(
        () =>
            find.text('Checkout · ready').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      retained = await call({
        'command': 'worktrees',
        'session': source,
        'request': {'action': 'list'},
      });
      final owned = (retained['items'] as List).single as Map;
      final root = Directory(owned['root'] as String);
      expect(root.isAbsolute, true);
      expect(
        root.resolveSymbolicLinksSync(),
        startsWith(
          '${data.parent.resolveSymbolicLinksSync()}${Platform.pathSeparator}',
        ),
      );
      expect(File('${root.path}/note.txt').readAsStringSync(), 'base\n');
      File('${root.path}/note.txt')
          .writeAsStringSync('independent task work\n');
      expect(
        File('${project.path}/note.txt').readAsStringSync(),
        'keep original dirty file\n',
      );
      expect(File('${project.path}/.git/index').readAsBytesSync(), index);
      // The managed checkout can transfer selected work while both chats retain
      // their context. First exercise the real conflict refusal, then explicitly
      // resolve only this public fixture's destination and review again.
      await tap('Transfer changes');
      await waitUntil(
        () =>
            find.text('Compare changes').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      await tap('Compare changes');
      await waitUntil(
        () =>
            find
                .widgetWithText(CheckboxListTile, 'note.txt')
                .evaluate()
                .isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      await t.ensureVisible(find.widgetWithText(CheckboxListTile, 'note.txt'));
      await t.pump();
      await t.tap(find.widgetWithText(CheckboxListTile, 'note.txt'));
      await t.pump();
      await tap('Review transfer (1)');
      await waitUntil(
        () =>
            find.text('Apply transfer').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(
        t
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Apply transfer'),
            )
            .onPressed,
        isNull,
      );
      expect(
        File('${project.path}/note.txt').readAsStringSync(),
        'keep original dirty file\n',
      );
      expect(
        File('${root.path}/note.txt').readAsStringSync(),
        'independent task work\n',
      );
      File('${project.path}/note.txt').writeAsStringSync('base\n');
      await tap('Change selection');
      await tap('Compare changes');
      await waitUntil(
        () => find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      await tap('Review transfer (1)');
      await waitUntil(
        () =>
            find.text('Apply transfer').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      await tap('Apply transfer');
      await waitUntil(
        () =>
            File('${project.path}/note.txt').readAsStringSync() ==
                'independent task work\n' &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(File('${project.path}/.git/index').readAsBytesSync(), index);
      expect(chat.draft, 'Keep my source draft 世界');
      await waitUntil(() {
        final close = find.widgetWithText(TextButton, 'Close');
        return close.evaluate().isNotEmpty &&
            t.widget<TextButton>(close.last).onPressed != null;
      });
      await t.scrollUntilVisible(
        find.textContaining('Selected changes transferred. Source checkout'),
        -200,
        scrollable: find
            .descendant(
              of: find.byType(WorktreeTransferInspector),
              matching: find.byType(Scrollable),
            )
            .first,
      );
      await t.pumpAndSettle();
      final transferBoundary =
          key.currentContext!.findRenderObject() as RenderRepaintBoundary;
      await t.runAsync(() async {
        final image = await transferBoundary.toImage();
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        File('${data.parent.path}/worktrees-transfer-native.png')
            .writeAsBytesSync(bytes!.buffer.asUint8List());
        image.dispose();
      });
      await tap('Close');
      await t.pumpAndSettle();
      // Archive/restore uses rendered reviews, with ignored data and staging
      // retained outside the owned checkout before its removal.
      File('${root.path}/.gitignore').writeAsStringSync('ignored/\n');
      Directory('${root.path}/ignored').createSync();
      File('${root.path}/ignored/native.bin')
          .writeAsBytesSync([0, 255, 1, 0, 254]);
      Directory('${root.path}/empty-retained').createSync();
      final ownedGit = owned['identity']['gitDirectory'] as String;
      final ownedIndex = File('$ownedGit/index').readAsBytesSync();
      await tap('Review archive');
      await waitUntil(
        () =>
            find.text('Archive checkout').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(root.existsSync(), true);
      await tap('Archive checkout');
      await waitUntil(
        () =>
            find.text('Checkout · archived').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(root.existsSync(), false);
      expect(chat.session, source);
      expect(chat.draft, 'Keep my source draft 世界');
      expect(
        File('${project.path}/note.txt').readAsStringSync(),
        'independent task work\n',
      );
      await tap('Review restore');
      await waitUntil(
        () =>
            find.text('Restore checkout').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(root.existsSync(), false);
      await tap('Restore checkout');
      await waitUntil(
        () =>
            find.text('Checkout · ready').evaluate().isNotEmpty &&
            find.byType(LinearProgressIndicator).evaluate().isEmpty,
      );
      expect(
        File('${root.path}/note.txt').readAsStringSync(),
        'independent task work\n',
      );
      expect(File('${root.path}/ignored/native.bin').readAsBytesSync(), [
        0,
        255,
        1,
        0,
        254,
      ]);
      expect(Directory('${root.path}/empty-retained').existsSync(), true);
      final current = await call({
        'command': 'worktrees',
        'session': owned['session'],
        'request': {'action': 'inspect'},
      }) as Map;
      expect(
        File('${current['identity']['gitDirectory']}/index').readAsBytesSync(),
        ownedIndex,
      );
      expect(File('${project.path}/.git/index').readAsBytesSync(), index);
      await waitUntil(() {
        final close = find.widgetWithText(TextButton, 'Close');
        return close.evaluate().isNotEmpty &&
            t.widget<TextButton>(close.last).onPressed != null;
      });
      await t.ensureVisible(find.text('Checkout · ready'));
      await t.pumpAndSettle();
      final boundary =
          key.currentContext!.findRenderObject() as RenderRepaintBoundary;
      await t.runAsync(() async {
        final image = await boundary.toImage();
        final bytes = await image.toByteData(format: ui.ImageByteFormat.png);
        File('${data.parent.path}/worktrees-native.png')
            .writeAsBytesSync(bytes!.buffer.asUint8List());
        image.dispose();
      });
      await tap('Open chat');
      await waitUntil(
        () =>
            chat.session == owned['session'] &&
            !chat.changing &&
            find.byType(WorktreesInspector).evaluate().isEmpty,
      );
      expect(chat.workspaceRoot, owned['root']);
      await t.runAsync(() => chat.select(source));
      expect(chat.draft, 'Keep my source draft 世界');
      expect(
        await call({'command': 'savedDraft', 'session': source}),
        chat.draft,
      );
      File('${data.parent.path}/worktrees-receipt.json').writeAsStringSync(
        jsonEncode({
          'scope': 'Rendered Flutter controls and normal native bridge; public Git fixture, no model or physical input qualification',
          'staleReviewRefused': true,
          'independentFiles': true,
          'sourceIndexRetained': true,
          'draftRetained': true,
          'transferConflictRefused': true,
          'selectedTransferApplied': true,
          'destinationIndexRetainedAfterTransfer': true,
          'archiveRemovedOnlyOwnedCheckout': true,
          'restorePreservedIgnoredBinaryAndEmptyFolder': true,
          'restorePreservedIndex': true,
          'worktree': owned,
          'passed': true,
        }),
      );
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
    },
    skip: Platform.environment['DOLORES_AUDIT_LIBRARY'] == null,
  );
}
