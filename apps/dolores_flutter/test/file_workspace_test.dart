import 'dart:io';
import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:re_editor/re_editor.dart';
import 'package:dolores_flutter/app_host.dart';
import 'package:dolores_flutter/chat.dart';
import 'package:dolores_flutter/file_workspace.dart';
import 'package:dolores_flutter/theme.dart';

import 'editor_test.dart' show EditingBridge;

void main() {
  setUpAll(() async {
    if (Platform.environment['DOLORES_UI_RENDERS'] == null) return;
    final directory =
        Platform.environment['DOLORES_UI_FONTS'] ?? 'C:/Windows/Fonts';
    for (final family in ['Roboto', 'Segoe UI']) {
      await ui.loadFontFromList(
        Uint8List.fromList(await File('$directory/segoeui.ttf').readAsBytes()),
        fontFamily: family,
      );
    }
    await ui.loadFontFromList(
      Uint8List.fromList(await File('$directory/consola.ttf').readAsBytes()),
      fontFamily: 'Consolas',
    );
    await ui.loadFontFromList(
      Uint8List.fromList(
        await File(
          '../../output/toolchains/flutter/bin/cache/artifacts/material_fonts/MaterialIcons-Regular.otf',
        ).readAsBytes(),
      ),
      fontFamily: 'MaterialIcons',
    );
  });
  testWidgets(
    'drag reorder, cancelled drag and edge split retain the shared buffer; dirty close can be cancelled',
    (t) async {
      t.view.physicalSize = const Size(1300, 800);
      t.view.devicePixelRatio = 1;
      addTearDown(t.view.resetPhysicalSize);
      addTearDown(t.view.resetDevicePixelRatio);
      final bridge = EditingBridge();
      final host = AppHost(ChatController(bridge)..loading = false);
      await host.files.bind('A', 'C:/A');
      final w = host.files.selected!;
      final a = (await host.files.open(w, 'a.txt'))!;
      w.layoutOwner.pin(w.layoutOwner.activeGroup, a.id);
      final b = (await host.files.open(w, 'b.txt'))!;
      w.layoutOwner.pin(w.layoutOwner.activeGroup, b.id);
      await t.pumpWidget(
        MaterialApp(
          home: Scaffold(body: FileWorkspaceView(host: host)),
        ),
      );
      await t.pumpAndSettle();
      final first = w.layoutOwner.activeGroup;
      Finder tab(String id) => find
          .descendant(
            of: find.byKey(Key('file-tab-$first-$id')).first,
            matching: find.byType(Text),
          )
          .first;
      await t.dragFrom(
        t.getCenter(tab(b.id)),
        t.getCenter(tab(a.id)) - t.getCenter(tab(b.id)),
      );
      await t.pumpAndSettle();
      expect(w.layoutOwner.active.tabs.first, b.id);
      final order = w.layoutOwner.active.tabs.toList();
      final cancelled = await t.startGesture(t.getCenter(tab(a.id)));
      await cancelled.moveBy(const Offset(40, 100));
      await t.pump();
      await cancelled.cancel();
      await t.pumpAndSettle();
      expect(w.layoutOwner.active.tabs, order);
      final gesture = await t.startGesture(t.getCenter(tab(a.id)));
      await gesture.moveBy(const Offset(40, 60));
      await t.pump();
      await gesture.moveTo(const Offset(1260, 350));
      await t.pump();
      await gesture.up();
      await t.pumpAndSettle();
      expect(w.layoutOwner.groups.length, 2);
      final editor = t.widget<CodeEditor>(find.byType(CodeEditor).last);
      editor.controller!.replaceSelection('mine ');
      await t.pump();
      await host.files.flush(a);
      expect(a.text, 'mine saved');
      expect(a.dirty, true);
      await t.pump(const Duration(milliseconds: 350));
      final active = w.layoutOwner.groups.values
          .singleWhere((g) => g.tabs.contains(a.id))
          .id;
      final close = find.descendant(
        of: find.byKey(Key('file-tab-$active-${a.id}')),
        matching: find.byType(IconButton),
      );
      await t.tap(close);
      await t.pumpAndSettle();
      expect(find.text('Keep editing'), findsOneWidget);
      await t.tap(find.text('Keep editing'));
      await t.pumpAndSettle();
      expect(host.files.documents[a.id], same(a));
      expect(a.dirty, true);
      bridge.failSave = true;
      await t.tap(
        find.descendant(
          of: find.byKey(Key('file-tab-$active-${a.id}')),
          matching: find.byTooltip('Close file'),
        ),
      );
      await t.pumpAndSettle();
      await t.tap(find.widgetWithText(TextButton, 'Save').last);
      await t.pumpAndSettle();
      expect(a.closed, false);
      expect(a.text, 'mine saved');
      expect(t.takeException(), isNull);
      await t.pumpWidget(const SizedBox());
      host.dispose();
    },
  );
  for (final dark in [false, true]) {
    testWidgets(
      'four file groups remain reachable in wide and compact ${dark ? 'dark' : 'light'} layouts',
      (t) async {
        t.view.physicalSize = const Size(1400, 900);
        t.view.devicePixelRatio = 1;
        addTearDown(t.view.resetPhysicalSize);
        addTearDown(t.view.resetDevicePixelRatio);
        final host = AppHost(ChatController(EditingBridge())..loading = false);
        await host.files.bind('A', 'C:/A');
        final w = host.files.selected!;
        final a = (await host.files.open(w, 'a.txt'))!;
        final original = w.layoutOwner.activeGroup;
        w.layoutOwner.split(original, a.id, Axis.horizontal);
        final right = w.layoutOwner.activeGroup;
        w.layoutOwner.split(original, a.id, Axis.vertical);
        w.layoutOwner.split(right, a.id, Axis.vertical);
        final boundary = GlobalKey();
        await t.pumpWidget(
          MaterialApp(
            theme: doloresTheme(dark),
            home: Scaffold(
              body: RepaintBoundary(
                key: boundary,
                child: FileWorkspaceView(host: host),
              ),
            ),
          ),
        );
        await t.pumpAndSettle();
        expect(find.byType(CodeEditor), findsNWidgets(4));
        expect(t.takeException(), isNull);
        Future<void> capture(String name) async {
          if (Platform.environment['DOLORES_UI_RENDERS'] == null) return;
          await t.runAsync(() async {
            final image =
                await (boundary.currentContext!.findRenderObject()
                        as RenderRepaintBoundary)
                    .toImage();
            final data = await image.toByteData(format: ui.ImageByteFormat.png);
            image.dispose();
            final file = File(
              '${Platform.environment['DOLORES_UI_RENDERS']}/$name.png',
            );
            await file.parent.create(recursive: true);
            await file.writeAsBytes(data!.buffer.asUint8List());
          });
        }

        await capture('folders-four-${dark ? 'dark' : 'light'}');
        t.view.physicalSize = const Size(420, 480);
        await t.pumpAndSettle();
        expect(find.byType(CodeEditor), findsOneWidget);
        expect(find.textContaining('Group 1'), findsOneWidget);
        await t.tap(find.textContaining('Group 1'));
        await t.pumpAndSettle();
        expect(w.layoutOwner.activeGroup, original);
        expect(t.takeException(), isNull);
        await capture('folders-compact-${dark ? 'dark' : 'light'}');
        await t.pumpWidget(const SizedBox());
        host.dispose();
      },
    );
  }
}
