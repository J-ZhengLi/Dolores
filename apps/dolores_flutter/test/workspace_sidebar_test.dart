import 'package:dolores_flutter/file_host.dart';
import 'package:dolores_flutter/folders.dart';
import 'package:dolores_flutter/git_host.dart';
import 'package:dolores_flutter/source_control.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import 'file_host_test.dart' show FileBridge;
import 'source_control_test.dart' show MutationBridge;
import 'frost_design_test.dart' show capture, viewport;
import 'support/render_fonts.dart';

import 'package:dolores_flutter/theme.dart';

Map<String, dynamic> entry(String path, bool directory) => {
  'name': path.split('/').last,
  'path': path,
  'directory': directory,
};

FileHost fileFixture() {
  final files = FileHost(FileBridge());
  files.selected = FileWorkspace('A', r'C:\Public\PROJECT_DOLORES', 'A')
    ..tree['.'] = (TreePage()
      ..cursor = null
      ..entries.addAll([
        entry('README.md', false),
        entry('assets', true),
        entry('AGENTS.md', false),
        entry('apps', true),
        entry('Cargo.toml', false),
        entry('docs', true),
      ]))
    ..tree['apps'] = (TreePage()
      ..cursor = null
      ..entries.addAll([
        entry('apps/z-last.dart', false),
        entry('apps/lib', true),
        entry('apps/a-first.dart', false),
      ]))
    ..expanded.add('apps');
  return files;
}

GitHost gitFixture({MutationBridge? bridge}) {
  final git = GitHost(bridge ?? MutationBridge());
  git.selected = GitWorkspace('C:/Public/PROJECT_DOLORES', 'A')
    ..status = {
      'root': 'C:/Public/PROJECT_DOLORES',
      'branch': 'main',
      'entries': [
        {
          'path': 'apps/dolores_flutter/lib/folders.dart',
          'index': ' ',
          'worktree': 'M',
        },
        {
          'path': 'apps/dolores_flutter/lib/theme.dart',
          'index': 'M',
          'worktree': ' ',
        },
        {
          'path': 'docs/a-very-long-document-name-with-Unicode-世界.md',
          'index': '?',
          'worktree': '?',
        },
      ],
    }
    ..history = [
      {
        'id': 'aaaaaaaa12345678',
        'subject': 'Refine the workspace navigation',
        'author': 'Fixture',
      },
      {
        'id': 'bbbbbbbb12345678',
        'subject': 'Preserve pending edits after a failed operation',
        'author': 'Fixture',
      },
    ];
  return git;
}

void main() {
  testWidgets(
    'tree sorts loaded siblings without changing pages or expansion',
    (t) async {
      final files = fileFixture();
      addTearDown(files.dispose);
      final w = files.selected!;
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 252,
              child: FolderTree(files: files, openFolder: () {}),
            ),
          ),
        ),
      );
      double y(String path) =>
          t.getTopLeft(find.byKey(ValueKey('file-tree-$path'))).dy;
      expect(y('apps'), lessThan(y('assets')));
      expect(y('assets'), lessThan(y('AGENTS.md')));
      expect(y('docs'), lessThan(y('README.md')));
      expect(y('apps/lib'), lessThan(y('apps/a-first.dart')));
      expect(y('apps/a-first.dart'), lessThan(y('apps/z-last.dart')));
      expect(
        t.getSize(find.byKey(const ValueKey('file-tree-apps'))).height,
        26,
      );
      expect(w.tree['.']!.entries.first['name'], 'README.md');
      // A later lazy page can contain folders; re-sort the loaded sibling set.
      w.tree['.']!.entries.add(entry('adapters', true));
      files.changed();
      await t.pump();
      expect(y('adapters'), lessThan(y('apps')));
      expect(w.expanded, contains('apps'));
      await t.tap(find.byTooltip('Collapse folders'));
      await t.pump();
      expect(find.byKey(const ValueKey('file-tree-apps/lib')), findsNothing);
      expect(w.expanded, {'.'});
      expect(w.tree['apps']!.entries, hasLength(3));
      await t.tap(find.byKey(const Key('files-project-root')));
      await t.pump();
      expect(find.byKey(const ValueKey('file-tree-README.md')), findsNothing);
      await t.tap(find.byKey(const Key('files-project-root')));
      await t.pump();
      expect(find.byKey(const ValueKey('file-tree-README.md')), findsOneWidget);
      expect(t.takeException(), isNull);
    },
  );

  testWidgets(
    'classic staging groups collapse independently and commit failures retain drafts',
    (t) async {
      final bridge = MutationBridge()..reject = true;
      final git = gitFixture(bridge: bridge);
      addTearDown(git.dispose);
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: SizedBox(
              width: 320,
              child: SourceControlPanel(git: git, openFolder: () {}),
            ),
          ),
        ),
      );
      expect(find.text('PROJECT_DOLORES'), findsNothing);
      expect(find.text('C:/Public/PROJECT_DOLORES'), findsNothing);
      expect(find.text('folders.dart'), findsOneWidget);
      expect(
        t
            .getSize(
              find.byKey(
                const ValueKey(
                  'git-working-apps/dolores_flutter/lib/folders.dart',
                ),
              ),
            )
            .height,
        26,
      );
      await t.tap(find.text('Staged Changes'));
      await t.pump();
      expect(find.text('theme.dart'), findsNothing);
      expect(find.text('folders.dart'), findsOneWidget);
      await t.tap(find.text('Staged Changes'));
      await t.pump();
      expect(find.text('theme.dart'), findsOneWidget);
      await t.enterText(
        find.byKey(const Key('git-commit-message')),
        'Keep my draft',
      );
      await t.pump();
      await t.tap(find.byKey(const Key('git-commit-button')));
      await t.pumpAndSettle();
      expect(git.selected!.commitDraft, 'Keep my draft');
      expect(find.textContaining('Commit hook refused'), findsOneWidget);
      expect(find.text('Keep my draft'), findsOneWidget);
      expect(find.text('theme.dart'), findsOneWidget);
      expect(t.takeException(), isNull);
    },
  );

  for (final dark in [false, true]) {
    for (final width in [252.0, 360.0]) {
      for (final scale in [1.0, 2.0]) {
        testWidgets(
          'classic sidebar renders at width=$width scale=$scale dark=$dark',
          (t) async {
            viewport(t, Size(width * 2, 760));
            if (const String.fromEnvironment('DESIGN_RENDER_DIR').isNotEmpty) {
              await t.runAsync(loadReviewFonts);
            }
            final files = fileFixture(), git = gitFixture();
            addTearDown(files.dispose);
            addTearDown(git.dispose);
            final key = GlobalKey();
            await t.pumpWidget(
              RepaintBoundary(
                key: key,
                child: MaterialApp(
                  debugShowCheckedModeBanner: false,
                  theme: doloresTheme(dark),
                  home: MediaQuery(
                    data: MediaQueryData(textScaler: TextScaler.linear(scale)),
                    child: Scaffold(
                      body: Row(
                        children: [
                          SizedBox(
                            width: width,
                            child: FolderTree(files: files, openFolder: () {}),
                          ),
                          SizedBox(
                            width: width,
                            child: SourceControlPanel(
                              git: git,
                              openFolder: () {},
                            ),
                          ),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            );
            await t.pumpAndSettle();
            expect(t.takeException(), isNull);
            await capture(
              t,
              key,
              'classic-${dark ? 'dark' : 'light'}-${width.toInt()}-${scale.toInt()}x',
            );
            await t.pumpWidget(const SizedBox());
          },
        );
      }
    }
  }
}
