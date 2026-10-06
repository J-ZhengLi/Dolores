import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:flutter_test/flutter_test.dart';
import 'package:dolores_flutter/image_clipboard.dart';

Uint8List bitmap({int width = 2, int height = 2}) {
  final bytes = Uint8List(70);
  final h = ByteData.sublistView(bytes);
  bytes[0] = 0x42;
  bytes[1] = 0x4d;
  h.setUint32(2, bytes.length, Endian.little);
  h.setUint32(10, 54, Endian.little);
  h.setUint32(14, 40, Endian.little);
  h.setInt32(18, width, Endian.little);
  h.setInt32(22, height, Endian.little);
  h.setUint16(26, 1, Endian.little);
  h.setUint16(28, 24, Endian.little);
  for (var i = 54; i < bytes.length; i++) {
    bytes[i] = 120;
  }
  return bytes;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();
  test('Native Windows BMP becomes a readable PNG snapshot', () async {
    final png = await normalizeClipboardImage(bitmap());
    expect(png.take(8), [137, 80, 78, 71, 13, 10, 26, 10]);
    final codec = await ui.instantiateImageCodec(png);
    final frame = await codec.getNextFrame();
    expect(frame.image.width, 2);
    expect(frame.image.height, 2);
    frame.image.dispose();
    codec.dispose();
  });
  test('Oversized dimensions and malformed clipboard bitmaps refuse before decoding', () async {
    await expectLater(
      normalizeClipboardImage(bitmap(width: 4097)),
      throwsStateError,
    );
    await expectLater(
      normalizeClipboardImage(bitmap(width: 4096, height: 2048)),
      throwsStateError,
    );
    await expectLater(
      normalizeClipboardImage(Uint8List.fromList([66, 77])),
      throwsStateError,
    );
    await expectLater(
      normalizeClipboardImage(Uint8List(maxClipboardImageBytes + 1)),
      throwsStateError,
    );
  });
}
