import 'package:dolores_flutter/bridge.dart';
import 'package:dolores_flutter/skill_draft.dart';
import 'package:dolores_flutter/theme.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

class DraftBridge implements ChatBridge {
  final commands = <Map<String, dynamic>>[];
  bool pending = false,
      cancelled = false,
      fail = false,
      tied = false,
      repairable = false,
      outputLimited = false;
  String running = '';
  @override
  Future<void> open() async {}
  @override
  Future<void> close() async {}
  @override
  Future<dynamic> call(Map<String, dynamic> command) async {
    commands.add(command);
    switch (command['command']) {
      case 'reviewSkillExamples':
        return {
          'token': 'sources',
          'settings': {'maxOutputTokens': 4096, 'timeoutSeconds': 90},
          'examples': [
            {
              'userId': 1,
              'messageId': 2,
              'request': 'Review code',
              'response': 'Use focused tests.',
            },
          ],
        };
      case 'generateSkillDraft':
      case 'evaluateSkillDraft':
        if (command['command'] == 'evaluateSkillDraft' &&
            (command['draft']['name'] as String).contains('_')) {
          throw 'Use a valid skill name.';
        }
        running = command['command'] as String;
        return null;
      case 'cancel':
        cancelled = true;
        return null;
      case 'poll':
        if (pending && !cancelled) return [];
        if (cancelled) {
          return [
            {
              'type': 'done',
              'error': 'Skill draft stopped. Nothing was saved.',
            },
          ];
        }
        if (fail) {
          return [
            {'type': 'done', 'error': 'Provider unavailable. Try again.'},
          ];
        }
        if (running == 'generateSkillDraft') {
          if (outputLimited) {
            return [
              {
                'type': 'done',
                'error': 'Skill draft reached its 4096-token output limit. Increase Draft output tokens and generate again. Nothing was saved.',
              },
            ];
          }
          return [
            {
              'type': 'done',
              'skillDraft': {
                'token': 'draft',
                'model': 'fixture',
                'warning': repairable ? 'Use a valid skill name.' : null,
                'draft': {
                  'name': repairable ? 'review_notes' : 'review',
                  'description': 'When reviewing code',
                  'instructions': 'Use focused tests.',
                  'evidence': [
                    {'messageId': 2, 'quote': 'Use focused tests.'},
                  ],
                },
              },
            },
          ];
        }
        return [
          {
            'type': 'done',
            'skillEvaluation': {
              'token': 'tested',
              'promotable': !tied,
              'evaluation': {
                'model': 'fixture',
                'reviewedAt': 1,
                'results': [
                  {
                    'trial': {
                      'prompt': 'Review a change',
                      'required': ['focused'],
                      'forbidden': [],
                    },
                    'baseline': tied ? 'focused' : 'ordinary',
                    'candidate': 'focused',
                  },
                ],
              },
            },
          },
        ];
      default:
        return null;
    }
  }
}

Future<void> open(WidgetTester tester, DraftBridge bridge, bool dark) async {
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
            onPressed: () => showDialog<bool>(
              context: context,
              builder: (_) => SkillDraftInspector(
                bridge: bridge,
                session: 'chat',
                scope: 'global',
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

Future<void> reveal(WidgetTester tester, String key) async {
  FocusManager.instance.primaryFocus?.unfocus();
  final scrollable = find
      .descendant(
        of: find.byKey(const Key('skill-draft-scroll')),
        matching: find.byType(Scrollable),
      )
      .first;
  tester.state<ScrollableState>(scrollable).position.jumpTo(0);
  await tester.pumpAndSettle();
  await tester.scrollUntilVisible(
    find.byKey(Key(key)),
    220,
    scrollable: find
        .descendant(
          of: find.byKey(const Key('skill-draft-scroll')),
          matching: find.byType(Scrollable),
        )
        .first,
  );
  await Scrollable.ensureVisible(
    tester.element(find.byKey(Key(key))),
    alignment: .5,
  );
  await tester.pumpAndSettle();
}

Future<void> generate(WidgetTester tester) async {
  await reveal(tester, 'skill-example-2');
  await tester.tap(find.byKey(const Key('skill-example-2')));
  await tester.pumpAndSettle();
  await tester.tap(find.byKey(const Key('generate-skill-draft')));
  await tester.pumpAndSettle();
}

void main() {
  testWidgets(
    'draft limits start from model settings and failure allows an explicit larger retry',
    (tester) async {
      final bridge = DraftBridge()..outputLimited = true;
      await open(tester, bridge, true);
      await reveal(tester, 'skill-draft-output-tokens');
      expect(
        tester
            .widget<TextField>(
              find.byKey(const Key('skill-draft-output-tokens')),
            )
            .controller!
            .text,
        '4096',
      );
      await reveal(tester, 'skill-draft-timeout');
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('skill-draft-timeout')))
            .controller!
            .text,
        '90',
      );
      await generate(tester);
      expect(
        find.textContaining('Increase Draft output tokens'),
        findsOneWidget,
      );
      expect(
        bridge.commands
            .where((c) => c['command'] == 'generateSkillDraft')
            .length,
        1,
      );
      expect(
        bridge.commands.where((c) => c['command'] == 'promoteSkillDraft'),
        isEmpty,
      );
      await reveal(tester, 'skill-draft-output-tokens');
      await tester.enterText(
        find.byKey(const Key('skill-draft-output-tokens')),
        '8192',
      );
      await reveal(tester, 'skill-draft-timeout');
      await tester.enterText(
        find.byKey(const Key('skill-draft-timeout')),
        '240',
      );
      bridge.outputLimited = false;
      FocusManager.instance.primaryFocus?.unfocus();
      await tester.tap(find.byKey(const Key('generate-skill-draft')));
      await tester.pumpAndSettle();
      final requests = bridge.commands
          .where((c) => c['command'] == 'generateSkillDraft')
          .toList();
      expect(requests.length, 2);
      expect(requests[0]['settings'], {
        'maxOutputTokens': 4096,
        'timeoutSeconds': 90,
      });
      expect(requests[1]['settings'], {
        'maxOutputTokens': 8192,
        'timeoutSeconds': 240,
      });
      expect(requests[1]['messageIds'], [2]);
      expect(requests[1]['token'], 'sources');
      expect(
        bridge.commands.where((c) => c['command'] == 'promoteSkillDraft'),
        isEmpty,
      );
    },
  );
  testWidgets(
    'invalid generation limits are refused locally without sending sources',
    (tester) async {
      final bridge = DraftBridge();
      await open(tester, bridge, false);
      await reveal(tester, 'skill-example-2');
      await tester.tap(find.byKey(const Key('skill-example-2')));
      for (final invalid in ['0', '32769', '3.5', '']) {
        await reveal(tester, 'skill-draft-output-tokens');
        await tester.enterText(
          find.byKey(const Key('skill-draft-output-tokens')),
          invalid,
        );
        FocusManager.instance.primaryFocus?.unfocus();
        await tester.tap(find.byKey(const Key('generate-skill-draft')));
        await tester.pumpAndSettle();
        expect(
          find.text('Draft output tokens must be between 1 and 32,768.'),
          findsOneWidget,
        );
      }
      await reveal(tester, 'skill-draft-output-tokens');
      await tester.enterText(
        find.byKey(const Key('skill-draft-output-tokens')),
        '8192',
      );
      await reveal(tester, 'skill-draft-timeout');
      await tester.enterText(
        find.byKey(const Key('skill-draft-timeout')),
        '901',
      );
      FocusManager.instance.primaryFocus?.unfocus();
      await tester.tap(find.byKey(const Key('generate-skill-draft')));
      await tester.pumpAndSettle();
      expect(
        find.text('Draft timeout must be between 1 and 900 seconds.'),
        findsOneWidget,
      );
      expect(
        bridge.commands.where((c) => c['command'] == 'generateSkillDraft'),
        isEmpty,
      );
    },
  );
  testWidgets(
    'invalid generated metadata stays editable and cannot be tested until corrected',
    (tester) async {
      final bridge = DraftBridge()..repairable = true;
      await open(tester, bridge, true);
      await generate(tester);
      expect(
        find.text(
          'Correct the draft before evaluation: Use a valid skill name.',
        ),
        findsOneWidget,
      );
      await tester.tap(find.byKey(const Key('evaluate-skill-draft')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('promote-skill-draft')))
            .onPressed,
        isNull,
      );
      await reveal(tester, 'draft-skill-name');
      await tester.enterText(
        find.byKey(const Key('draft-skill-name')),
        'review-notes',
      );
      await tester.pump();
      await tester.tap(find.byKey(const Key('evaluate-skill-draft')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('promote-skill-draft')))
            .onPressed,
        isNotNull,
      );
      expect(
        bridge.commands
            .where((c) => c['command'] == 'generateSkillDraft')
            .length,
        1,
      );
    },
  );
  for (final dark in [false, true]) {
    testWidgets(
      'review, edit, test, invalidate and explicit promotion in compact ${dark ? 'dark' : 'light'} theme',
      (tester) async {
        final bridge = DraftBridge();
        await open(tester, bridge, dark);
        expect(bridge.commands.single['command'], 'reviewSkillExamples');
        expect(
          tester
              .widget<FilledButton>(
                find.byKey(const Key('generate-skill-draft')),
              )
              .onPressed,
          isNull,
        );
        await generate(tester);
        expect(
          bridge.commands.where((c) => c['command'] == 'promoteSkillDraft'),
          isEmpty,
        );
        expect(
          tester
              .widget<FilledButton>(
                find.byKey(const Key('promote-skill-draft')),
              )
              .onPressed,
          isNull,
        );
        await reveal(tester, 'draft-skill-instructions');
        await tester.enterText(
          find.byKey(const Key('draft-skill-instructions')),
          'Use focused tests and report failures.',
        );
        await reveal(tester, 'skill-test-prompt-0');
        await tester.enterText(
          find.byKey(const Key('skill-test-prompt-0')),
          'Review a change',
        );
        await reveal(tester, 'skill-test-required-0');
        await tester.enterText(
          find.byKey(const Key('skill-test-required-0')),
          'focused',
        );
        await tester.tap(find.byKey(const Key('evaluate-skill-draft')));
        await tester.pumpAndSettle();
        final submitted = bridge.commands.lastWhere(
          (c) => c['command'] == 'evaluateSkillDraft',
        );
        expect(
          submitted['draft']['instructions'],
          'Use focused tests and report failures.',
        );
        expect(submitted['trials'][0]['required'], ['focused']);
        expect(
          tester
              .widget<FilledButton>(
                find.byKey(const Key('promote-skill-draft')),
              )
              .onPressed,
          isNotNull,
        );
        await reveal(tester, 'skill-evaluation-receipt');
        expect(find.text('Baseline 0/1 · With draft 1/1'), findsOneWidget);
        await reveal(tester, 'draft-skill-name');
        await tester.enterText(
          find.byKey(const Key('draft-skill-name')),
          'review-changes',
        );
        await tester.pump();
        expect(
          tester
              .widget<FilledButton>(
                find.byKey(const Key('promote-skill-draft')),
              )
              .onPressed,
          isNull,
        );
        await tester.tap(find.byKey(const Key('evaluate-skill-draft')));
        await tester.pumpAndSettle();
        await tester.tap(find.byKey(const Key('promote-skill-draft')));
        await tester.pumpAndSettle();
        expect(
          bridge.commands
              .where((c) => c['command'] == 'promoteSkillDraft')
              .single['token'],
          'tested',
        );
        expect(find.byType(SkillDraftInspector), findsNothing);
        expect(tester.takeException(), isNull);
      },
    );
  }
  testWidgets(
    'a tied score cannot activate; a failed re-evaluation invalidates a previous pass and retains edits',
    (tester) async {
      final bridge = DraftBridge()..tied = true;
      await open(tester, bridge, true);
      await generate(tester);
      await tester.tap(find.byKey(const Key('evaluate-skill-draft')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('promote-skill-draft')))
            .onPressed,
        isNull,
      );
      bridge.tied = false;
      await tester.tap(find.byKey(const Key('evaluate-skill-draft')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('promote-skill-draft')))
            .onPressed,
        isNotNull,
      );
      bridge.fail = true;
      await tester.tap(find.byKey(const Key('evaluate-skill-draft')));
      await tester.pumpAndSettle();
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('promote-skill-draft')))
            .onPressed,
        isNull,
      );
      await reveal(tester, 'draft-skill-name');
      expect(
        tester
            .widget<TextField>(find.byKey(const Key('draft-skill-name')))
            .controller!
            .text,
        'review',
      );
      await tester.tap(find.text('Close'));
      await tester.pumpAndSettle();
      expect(bridge.commands.last['command'], 'discardSkillDraft');
    },
  );
  testWidgets(
    'pending generation locks closing, duplicate requests and selection while keeping Stop available',
    (tester) async {
      final bridge = DraftBridge();
      await open(tester, bridge, false);
      await reveal(tester, 'skill-example-2');
      await tester.tap(find.byKey(const Key('skill-example-2')));
      await tester.pumpAndSettle();
      bridge.pending = true;
      await tester.tap(find.byKey(const Key('generate-skill-draft')));
      await tester.pump(const Duration(milliseconds: 150));
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('generate-skill-draft')))
            .onPressed,
        isNull,
      );
      expect(
        tester
            .widget<CheckboxListTile>(find.byKey(const Key('skill-example-2')))
            .onChanged,
        isNull,
      );
      await tester.tap(find.byKey(const Key('stop-skill-draft')));
      await tester.pumpAndSettle();
      expect(bridge.commands.where((c) => c['command'] == 'cancel').length, 1);
      expect(
        bridge.commands
            .where((c) => c['command'] == 'generateSkillDraft')
            .length,
        1,
      );
      expect(find.byKey(const Key('promote-skill-draft')), findsNothing);
      expect(
        tester
            .widget<TextButton>(find.widgetWithText(TextButton, 'Close'))
            .onPressed,
        isNotNull,
      );
      expect(tester.takeException(), isNull);
    },
  );
}
