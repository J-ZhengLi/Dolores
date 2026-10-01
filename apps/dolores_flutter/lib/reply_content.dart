import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_markdown_plus/flutter_markdown_plus.dart';
import 'package:markdown/markdown.dart' as md;

import 'theme.dart';

/// Native Markdown only: no HTML interpreter, image I/O, or automatic navigation.
class ReplyContent extends StatefulWidget {
  final String text;
  final bool streaming;
  final VoidCallback? onRendered;
  const ReplyContent({
    super.key,
    required this.text,
    this.streaming = false,
    this.onRendered,
  });

  @override
  State<ReplyContent> createState() => _ReplyContentState();
}

class _ReplyContentState extends State<ReplyContent> {
  static const interval = Duration(milliseconds: 80);
  String displayed = '';
  Timer? timer;
  Widget? cached;

  @override
  void initState() {
    super.initState();
    displayed = widget.text;
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    cached = null;
  }

  @override
  void didUpdateWidget(ReplyContent oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.text == displayed) {
      timer?.cancel();
      timer = null;
      return;
    }
    if (!widget.streaming || !oldWidget.streaming) {
      timer?.cancel();
      timer = null;
      displayed = widget.text;
      cached = null;
    } else {
      // Coalesce active deltas; completed replies reuse their parsed widget.
      timer ??= Timer(interval, () {
        timer = null;
        setState(() {
          displayed = widget.text;
          cached = null;
        });
        widget.onRendered?.call();
      });
    }
  }

  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (cached != null) return cached!;
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    final body = TextStyle(fontSize: 14, height: 1.65, color: p.text);
    // Bound rich layout cost without hiding any of the original reply.
    if (displayed.length > 32768 || '\n'.allMatches(displayed).length > 800) {
      return cached = Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            'Long reply shown as plain text.',
            style: TextStyle(color: p.muted, fontSize: 12),
          ),
          const SizedBox(height: 8),
          SelectableText(displayed, style: body),
        ],
      );
    }
    return cached = MarkdownBody(
      data: displayed,
      selectable: true,
      fitContent: false,
      extensionSet: md.ExtensionSet.gitHubFlavored,
      blockSyntaxes: const [_LiteralHtmlBlock()],
      onTapLink: (_, href, _) => showReplyLink(context, href),
      // Override all image sources, including local files, assets and data URIs.
      imageBuilder: (_, _, alt) => Text(
        'Image: ${alt?.isNotEmpty == true ? alt : 'not loaded'}',
        style: TextStyle(color: p.muted, fontSize: 13),
      ),
      builders: {'pre': _CodeBuilder()},
      styleSheet: MarkdownStyleSheet(
        p: body,
        a: body.copyWith(color: p.accent, decoration: TextDecoration.underline),
        h1: body.copyWith(
          fontSize: 24,
          height: 1.3,
          fontWeight: FontWeight.w600,
        ),
        h2: body.copyWith(
          fontSize: 20,
          height: 1.35,
          fontWeight: FontWeight.w600,
        ),
        h3: body.copyWith(
          fontSize: 17,
          height: 1.4,
          fontWeight: FontWeight.w600,
        ),
        h4: body.copyWith(fontWeight: FontWeight.w600),
        h5: body.copyWith(fontWeight: FontWeight.w600),
        h6: body.copyWith(color: p.muted, fontWeight: FontWeight.w600),
        code: body.copyWith(
          fontFamily: UiTokens.codeFont,
          fontFamilyFallback: UiTokens.codeFontFallback,
          backgroundColor: p.sidebar,
          fontSize: 13,
        ),
        blockSpacing: 12,
        listIndent: 24,
        listBullet: body,
        blockquote: body.copyWith(color: p.muted),
        blockquotePadding: const EdgeInsets.all(12),
        blockquoteDecoration: BoxDecoration(
          color: p.sidebar,
          border: Border(left: BorderSide(color: p.border, width: 3)),
        ),
        codeblockPadding: EdgeInsets.zero,
        codeblockDecoration: const BoxDecoration(),
        tableHead: body.copyWith(fontWeight: FontWeight.w600),
        tableBody: body,
        tableBorder: TableBorder.all(color: p.border),
        tableColumnWidth: const IntrinsicColumnWidth(),
        tableCellsPadding: const EdgeInsets.symmetric(
          horizontal: 12,
          vertical: 8,
        ),
        tableScrollbarThumbVisibility: true,
        horizontalRuleDecoration: BoxDecoration(
          border: Border(top: BorderSide(color: p.border)),
        ),
      ),
    );
  }
}

// The renderer drops top-level raw HTML text nodes by default. Wrap them in a
// native paragraph so examples stay visible, with no inline HTML interpretation.
class _LiteralHtmlBlock extends md.HtmlBlockSyntax {
  const _LiteralHtmlBlock();
  @override
  md.Node parse(md.BlockParser parser) =>
      md.Element('p', [super.parse(parser)]);
}

class _CodeBuilder extends MarkdownElementBuilder {
  @override
  Widget? visitElementAfterWithContext(
    BuildContext context,
    md.Element element,
    TextStyle? preferredStyle,
    TextStyle? parentStyle,
  ) {
    final code = element.children?.whereType<md.Element>().firstOrNull;
    final language = code?.attributes['class']?.replaceFirst('language-', '');
    return ReplyCodeBlock(text: element.textContent, language: language);
  }
}

class ReplyCodeBlock extends StatefulWidget {
  final String text;
  final String? language;
  const ReplyCodeBlock({super.key, required this.text, this.language});

  @override
  State<ReplyCodeBlock> createState() => _ReplyCodeBlockState();
}

class _ReplyCodeBlockState extends State<ReplyCodeBlock> {
  final scroll = ScrollController();
  @override
  void dispose() {
    scroll.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final p = Palette(Theme.of(context).brightness == Brightness.dark);
    return Container(
      width: double.infinity,
      decoration: BoxDecoration(
        color: p.sidebar,
        border: Border.all(color: p.border),
        borderRadius: BorderRadius.circular(8),
      ),
      clipBehavior: Clip.hardEdge,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Padding(
            padding: const EdgeInsets.only(left: 12, right: 4),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    widget.language?.isNotEmpty == true
                        ? widget.language!
                        : 'Code',
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                    style: TextStyle(fontSize: 12, color: p.muted),
                  ),
                ),
                TextButton.icon(
                  onPressed: () async {
                    await Clipboard.setData(ClipboardData(text: widget.text));
                    if (context.mounted) {
                      ScaffoldMessenger.of(context).showSnackBar(
                        const SnackBar(content: Text('Code copied.')),
                      );
                    }
                  },
                  icon: const Icon(Icons.copy_outlined, size: 14),
                  label: const Text('Copy code'),
                ),
              ],
            ),
          ),
          Divider(height: 1, color: p.border),
          Scrollbar(
            controller: scroll,
            thumbVisibility: true,
            child: SingleChildScrollView(
              controller: scroll,
              scrollDirection: Axis.horizontal,
              padding: const EdgeInsets.all(12),
              child: SelectableText(
                widget.text,
                style: TextStyle(
                  fontFamily: UiTokens.codeFont,
                  fontFamilyFallback: UiTokens.codeFontFallback,
                  fontSize: 13,
                  height: 1.6,
                  color: p.text,
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

Future<void> showReplyLink(BuildContext context, String? href) async {
  final uri = href == null ? null : Uri.tryParse(href);
  if (uri == null ||
      href!.length > 2048 ||
      RegExp(r'[\x00-\x20\x7f\u202a-\u202e\u2066-\u2069]').hasMatch(href) ||
      !['http', 'https'].contains(uri.scheme) ||
      uri.host.isEmpty ||
      uri.userInfo.isNotEmpty) {
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Only HTTP and HTTPS links are supported.')),
    );
    return;
  }
  await showDialog<void>(
    context: context,
    builder: (context) => AlertDialog(
      title: const Text('Link destination'),
      content: SingleChildScrollView(child: SelectableText(href)),
      actions: [
        TextButton(
          onPressed: () => Navigator.pop(context),
          child: const Text('Close'),
        ),
        TextButton(
          onPressed: () async {
            await Clipboard.setData(ClipboardData(text: href));
            if (context.mounted) Navigator.pop(context);
          },
          child: const Text('Copy link'),
        ),
      ],
    ),
  );
}
