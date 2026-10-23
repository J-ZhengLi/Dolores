import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:dolores_flutter/worktree_transfer.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';
import 'worktrees_test.dart' show WorktreeBridge, controller, record;

class TransferBridge extends WorktreeBridge {
  Map? transfer;
  String? action;
  bool conflicts = false, failApply = false, uncertainStatus = false;
  int applies = 0;
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    final request = command['request'];
    if (command['command'] == 'worktrees' && request is Map) {
      commands.add(command);
      switch (request['action']) {
        case 'listTransfers':
          return {
            'items': transfer == null
                ? []
                : [
                    {
                      'receipt': transfer!['receipt'],
                      'directory': transfer!['directory'],
                      'result': {
                        'state': applies > 0
                            ? (failApply ? 'uncertain' : 'applied')
                            : (conflicts ? 'conflict' : 'reviewed'),
                        'notice': applies > 0
                            ? (failApply
                                  ? 'Publication interrupted. Inspect retained versions before retrying.'
                                  : 'Selected changes transferred.')
                            : transfer!['receipt']['notice'],
                      },
                    },
                  ],
          };
        case 'compareTransfer':
          action = 'compareTransfer';
          return {'job': 'compare'};
        case 'reviewTransfer':
          action = 'reviewTransfer';
          transfer = {
            'token': 'transfer-token',
            'directory': 'C:/Dolores/worktree-transfers/public-fixture',
            'receipt': {
              'source': record(),
              'destinationRoot': 'C:/projects/public-fixture',
              'paths': request['paths'],
              'applicable': !conflicts,
              'notice': conflicts
                  ? 'Transfer conflicts with the destination. Neither checkout was changed.'
                  : 'Ready to transfer selected saved changes.',
            },
            'patch': '--- a/note.txt\n+++ b/note.txt\n@@ -1 +1 @@\n-base\n+selected change\n',
          };
          return {'job': 'review'};
        case 'applyTransfer':
          action = 'applyTransfer';
          applies++;
          return {'job': 'apply'};
        case 'discardReview':
          return null;
        case 'cancel':
          return null;
        case 'poll':
          if (action == 'applyTransfer' && uncertainStatus) {
            uncertainStatus = false;
            throw StateError(
              'Status unavailable. Check status; do not repeat transfer.',
            );
          }
          return {
            'done': true,
            'value': switch (action) {
              'compareTransfer' => {
                'paths': ['note.txt', 'new.bin'],
              },
              'reviewTransfer' => transfer,
              _ => {
                'applied': !failApply,
                'notice': failApply
                    ? 'Publication interrupted. Inspect retained versions before retrying.'
                    : 'Selected changes transferred.',
              },
            },
          };
      }
    }
    return super.call(command);
  }
}

Future<ChatController> panel(
  WidgetTester t,
  TransferBridge bridge, {
  bool dark = false,
  GlobalKey? key,
}) async {
  final chat = controller(bridge);
  chat.sessions = [
    {
      'id': 'destination',
      'title': 'Original task',
      'updatedAt': 1,
      'workspace': {'kind': 'project', 'root': 'C:/projects/public-fixture'},
    },
  ];
  await t.pumpWidget(
    RepaintBoundary(
      key: key,
      child: MaterialApp(
        theme: doloresTheme(dark),
        home: Scaffold(
          body: Builder(
            builder: (context) => TextButton(
              onPressed: () => showWorktreeTransfer(context, chat, record()),
              child: const Text('Transfer'),
            ),
          ),
        ),
      ),
    ),
  );
  await t.tap(find.text('Transfer'));
  await t.pumpAndSettle();
  return chat;
}

Future<void> tap(WidgetTester t, String label, {bool settle = true}) async {
  await t.ensureVisible(find.text(label).first);
  await t.pump();
  await t.tap(find.text(label).first);
  if (settle) {
    await t.pumpAndSettle();
  } else {
    await t.pump();
    await t.pump();
  }
}

Future<void> prepare(WidgetTester t) async {
  await tap(t, 'Compare changes');
  await tap(t, 'note.txt');
  await tap(t, 'Review transfer (1)');
}

void main() {
  testWidgets(
    'selection is reviewed before one explicit transfer and source draft remains',
    (t) async {
      final bridge = TransferBridge();
      final chat = await panel(t, bridge);
      await prepare(t);
      expect(bridge.applies, 0);
      expect(find.text('C:/projects/public-fixture'), findsOneWidget);
      final review = bridge.commands.singleWhere(
        (c) =>
            c['command'] == 'worktrees' &&
            c['request']['action'] == 'reviewTransfer',
      );
      expect(review['session'], 'owned');
      expect(review['request']['paths'], ['note.txt']);
      expect(review['request']['destination'], 'destination');
      await tap(t, 'Apply transfer');
      expect(bridge.applies, 1);
      expect(
        find.textContaining('Selected changes transferred.'),
        findsWidgets,
      );
      await t.scrollUntilVisible(
        find.text('Transfer · applied'),
        200,
        scrollable: find
            .descendant(
              of: find.byType(WorktreeTransferInspector),
              matching: find.byType(Scrollable),
            )
            .first,
      );
      expect(find.text('Transfer · applied'), findsOneWidget);
      expect(chat.session, 'source');
      expect(chat.draft, 'Keep my unsent source draft');
      expect(bridge.commands.where((c) => c['command'] == 'start'), isEmpty);
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'conflict disables Apply and exposes both retained versions without replay',
    (t) async {
      final bridge = TransferBridge()..conflicts = true;
      final chat = await panel(t, bridge);
      await prepare(t);
      expect(
        t
            .widget<FilledButton>(
              find.widgetWithText(FilledButton, 'Apply transfer'),
            )
            .onPressed,
        isNull,
      );
      await tap(t, 'Transfer · conflict');
      expect(
        find.text('C:/Dolores/worktree-transfers/public-fixture'),
        findsWidgets,
      );
      expect(find.text('Copy recovery folder'), findsOneWidget);
      expect(bridge.applies, 0);
      await tap(t, 'Close');
      expect(find.byType(WorktreeTransferInspector), findsNothing);
      expect(chat.draft, 'Keep my unsent source draft');
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  testWidgets(
    'uncertain response checks the same apply job and retains failure evidence',
    (t) async {
      final bridge = TransferBridge()
        ..uncertainStatus = true
        ..failApply = true;
      final chat = await panel(t, bridge);
      await prepare(t);
      await tap(t, 'Apply transfer', settle: false);
      for (
        var i = 0;
        i < 20 && find.textContaining('Status unavailable.').evaluate().isEmpty;
        i++
      ) {
        await t.pump(const Duration(milliseconds: 50));
      }
      expect(find.textContaining('Status unavailable.'), findsOneWidget);
      expect(
        t
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      await tap(t, 'Check status');
      expect(bridge.applies, 1);
      expect(find.textContaining('Publication interrupted.'), findsWidgets);
      await t.scrollUntilVisible(
        find.text('Transfer · uncertain'),
        200,
        scrollable: find
            .descendant(
              of: find.byType(WorktreeTransferInspector),
              matching: find.byType(Scrollable),
            )
            .first,
      );
      expect(find.text('Transfer · uncertain'), findsOneWidget);
      expect(chat.draft, 'Keep my unsent source draft');
      await t.pumpWidget(const SizedBox());
      chat.dispose();
    },
  );

  for (final dark in [false, true]) {
    for (final width in [420.0, 1100.0]) {
      testWidgets('transfer conflict review render dark=$dark width=$width', (
        t,
      ) async {
        viewport(t, Size(width, 900));
        await t.runAsync(loadReviewFonts);
        final bridge = TransferBridge()..conflicts = true;
        final key = GlobalKey();
        final chat = await panel(t, bridge, dark: dark, key: key);
        await prepare(t);
        expect(t.takeException(), isNull);
        await capture(
          t,
          key,
          'transfer-conflict-${dark ? 'dark' : 'light'}-${width.toInt()}',
        );
        await t.pumpWidget(const SizedBox());
        chat.dispose();
      });
    }
  }
}
