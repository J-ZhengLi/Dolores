import 'package:flutter/material.dart';

import 'file_host.dart';
import 'file_editor.dart';
import 'app_host.dart';

Future<String?> filePathDialog(
  BuildContext context,
  String title, {
  String initial = '',
  String label='Relative path in this project',
}) async {
  final result = await showDialog<String>(
    context: context,
    builder: (context) => _PathDialog(title:title,initial:initial,label:label),
  );
  return result == null || result.trim().isEmpty ? null : result.trim();
}
class _PathDialog extends StatefulWidget {
  final String title,initial,label;
  const _PathDialog({required this.title,required this.initial,required this.label});
  @override State<_PathDialog> createState()=>_PathDialogState();
}
class _PathDialogState extends State<_PathDialog> {
  late final input=TextEditingController(text:widget.initial);
  @override void dispose(){input.dispose();super.dispose();}
  @override Widget build(BuildContext context)=>AlertDialog(title:Text(widget.title),
    content:TextField(controller:input,autofocus:true,decoration:InputDecoration(labelText:widget.label),onSubmitted:(v)=>Navigator.pop(context,v)),
    actions:[TextButton(onPressed:()=>Navigator.pop(context),child:const Text('Cancel')),TextButton(onPressed:()=>Navigator.pop(context,input.text),child:const Text('Continue'))]);
}

Future<void> quickOpen(BuildContext context,FileHost files) async {
  final w=files.selected;if(w==null)return;
  final path=await filePathDialog(context,'Quick open',label:'Relative file path in this project');
  if(path!=null)await files.open(w,path);
}

class FolderTree extends StatelessWidget {
  final FileHost files;
  final VoidCallback openFolder;
  const FolderTree({super.key, required this.files, required this.openFolder});
  List<(Map<String,dynamic>?,String,int)> rows(FileWorkspace w, String path, int depth) {
    final page = w.tree[path];
    if (page == null) return [];
    return [
      for (final entry in page.entries) ...[
        (entry,path,depth),
        if (entry['directory'] == true && w.expanded.contains(entry['path']))
          ...rows(w, entry['path'] as String, depth + 1),
      ],
      (null,path,depth),
    ];
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: files,
    builder: (context, _) {
      final w = files.selected;
      final items=w==null?< (Map<String,dynamic>?,String,int)>[]:rows(w,'.',0);
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                const Expanded(
                  child: Text('Folders',maxLines:1,overflow:TextOverflow.ellipsis,style: TextStyle(fontSize: 18)),
                ),
                if(w!=null)IconButton(tooltip:'Quick open (Ctrl+P)',onPressed:()=>quickOpen(context,files),icon:const Icon(Icons.search,size:18)),
                if (w != null)
                  IconButton(
                    tooltip: 'New file',
                    onPressed: () async {
                      final path = await filePathDialog(context, 'Create file');
                      if (path == null) return;
                      await files.open(w, path, action: 'create');
                      await files.load(w, '.', refresh: true);
                    },
                    icon: const Icon(Icons.note_add_outlined, size: 18),
                  ),
                if (w != null)
                  IconButton(
                    tooltip: 'Refresh files',
                    onPressed: () async {await files.refreshDocuments(w);await files.load(w, '.', refresh: true);},
                    icon: const Icon(Icons.refresh, size: 18),
                  ),
              ],
            ),
          ),
          if (w == null)
            TextButton(onPressed: openFolder, child: const Text('Open folder')),
          if (w != null)
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12),
              child: Text(w.root, maxLines: 2, overflow: TextOverflow.ellipsis),
            ),
          if (files.loading) const LinearProgressIndicator(),
          if (w != null)
            Expanded(
              child: ListView.builder(itemCount:w.recovery.length+items.length,itemBuilder:(context,index){
                if(index<w.recovery.length){final path=w.recovery[index];return ListTile(title:Text('Recover $path'),leading:const Icon(Icons.restore),onTap:()=>files.open(w,path,action:'recover'));}
                final (entry,path,depth)=items[index-w.recovery.length];
                if(entry==null){final page=w.tree[path]!;return Column(children:[if(page.error!=null)Text(page.error!),if(page.pending)const LinearProgressIndicator(),if(page.cursor!=null)TextButton(onPressed:page.pending?null:()=>files.load(w,path),child:Text(page.error==null?'Load more':'Retry'))]);}
                final directory = entry['directory'] == true;
                final entryPath = entry['path'] as String;
                return Semantics(
                  button: true,
                  child: InkWell(
                    onTap: () => directory
                        ? files.expand(w, entryPath)
                        : files.open(w, entryPath),
                    onDoubleTap: directory
                        ? null
                        : () => files.open(w, entryPath, preview: false),
                    child: ListTile(
                      dense: true,
                      contentPadding: EdgeInsets.only(
                        left: 8 + depth.clamp(0, 12) * 12,
                        right: 4,
                      ),
                      leading: Icon(
                        directory
                            ? (w.expanded.contains(entryPath)
                                  ? Icons.folder_open_outlined
                                  : Icons.folder_outlined)
                            : Icons.description_outlined,
                        size: 16,
                      ),
                      title: Text(
                        entry['name'] as String,
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ),
                );
              }),
            ),
        ],
      );
    },
  );
}

class FolderPreview extends StatelessWidget {
  final AppHost host;
  final FileHost files;
  const FolderPreview({super.key, required this.files, required this.host});
  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: files,
    builder: (context, _) {
      final d = files.active;
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          if (files.error != null)
            Padding(
              padding: const EdgeInsets.all(16),
              child: Text(files.error!),
            ),
          if (d != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Row(
                children: [
                  Expanded(child: Text(d.path)),
                  PopupMenuButton<String>(
                    tooltip: 'File actions',
                    onSelected: (action) async {
                      final w = files.selected!;
                      if (action == 'rename') {
                        final path = await filePathDialog(
                          context,
                          'Rename file',
                          initial: d.path,
                        );
                        if (path != null) {
                          await files.action(w, d, 'rename', path: path);
                        }
                      } else {
                        final confirmed = await showDialog<bool>(
                          context: context,
                          builder: (context) => AlertDialog(
                            title: Text('Delete ${d.path}?'),
                            content: const Text(
                              'This deletes the saved file from this project. Dirty documents must be saved or closed first.',
                            ),
                            actions: [
                              TextButton(
                                onPressed: () => Navigator.pop(context, false),
                                child: const Text('Cancel'),
                              ),
                              TextButton(
                                onPressed: () => Navigator.pop(context, true),
                                child: const Text('Delete'),
                              ),
                            ],
                          ),
                        );
                        if (confirmed == true) {
                          await files.action(w, d, 'delete');
                        }
                      }
                    },
                    itemBuilder: (_) => const [
                      PopupMenuItem(value: 'rename', child: Text('Rename')),
                      PopupMenuItem(value: 'delete', child: Text('Delete')),
                    ],
                  ),
                  IconButton(
                    tooltip: 'Close file',
                    onPressed: () => files.close(files.selected!, d),
                    icon: const Icon(Icons.close, size: 18),
                  ),
                ],
              ),
            ),
          if (d?.snapshot['reason'] != null)
            Padding(
              padding: const EdgeInsets.all(12),
              child: Text(
                '${d!.snapshot['reason']} Read-only preview; at most 64 KiB and 2,000 characters per displayed line.',
              ),
            ),
          Expanded(
            child: d == null
                ? const Center(child: Text('Select a file in the folder tree.'))
                : FileEditor(
                    key: ValueKey(d.id),
                    host: host,
                    workspace: files.selected!,
                    document: d,
                  ),
          ),
        ],
      );
    },
  );
}
