import 'dart:typed_data';
import 'dart:ui' as ui;

import 'package:pasteboard/pasteboard.dart';

const maxClipboardImageBytes = 2 * 1024 * 1024;

/// Read only on explicit paste. No clipboard polling or file/text fallback.
Future<Uint8List?> readClipboardImage() async {
  final bytes = await Pasteboard.image.timeout(const Duration(seconds: 10));
  return bytes == null ? null : normalizeClipboardImage(bytes);
}

/// Windows supplies BMP pixels; the stored/provider formats are PNG/JPEG.
/// Inspect dimensions before decoding, and bound both raw and encoded bytes.
Future<Uint8List> normalizeClipboardImage(Uint8List bytes) async {
  if (bytes.length >= 2 && bytes[0] == 0x42 && bytes[1] == 0x4d) {
    if (bytes.length < 54 || bytes.length > 16 * 1024 * 1024 + 65536) {
      throw StateError('Clipboard bitmap is invalid or too large.');
    }
    final header = ByteData.sublistView(bytes);
    final width = header.getInt32(18, Endian.little);
    final height = header.getInt32(22, Endian.little).abs();
    if (header.getUint32(14, Endian.little) < 40 ||
        width <= 0 ||
        height <= 0 ||
        width > 4096 ||
        height > 4096 ||
        width * height > 4 * 1024 * 1024) {
      throw StateError(
        'Image exceeds 4096 pixels per side or 4 megapixels. Resize it before pasting.',
      );
    }
    final codec = await ui.instantiateImageCodec(bytes);
    try {
      final frame = await codec.getNextFrame();
      try {
        final png = await frame.image.toByteData(
          format: ui.ImageByteFormat.png,
        );
        if (png == null || png.lengthInBytes > maxClipboardImageBytes) {
          throw StateError('Image exceeds 2 MiB. Resize it before pasting.');
        }
        return png.buffer.asUint8List(png.offsetInBytes, png.lengthInBytes);
      } finally {
        frame.image.dispose();
      }
    } finally {
      codec.dispose();
    }
  }
  if (bytes.length > maxClipboardImageBytes) {
    throw StateError('Image exceeds 2 MiB. Resize it before pasting.');
  }
  return bytes;
}
