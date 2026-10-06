import 'package:flutter/material.dart';

import 'file_host.dart';

Future<String?> filePathDialog(BuildContext context,String title,{String initial=''}) async {
  final input=TextEditingController(text:initial);
  final result=await showDialog<String>(context:context,builder:(context)=>AlertDialog(title:Text(title),
    content:TextField(controller:input,autofocus:true,decoration:const InputDecoration(labelText:'Relative path in this project'),onSubmitted:(v)=>Navigator.pop(context,v)),
    actions:[TextButton(onPressed:()=>Navigator.pop(context),child:const Text('Cancel')),TextButton(onPressed:()=>Navigator.pop(context,input.text),child:const Text('Continue'))]));
  input.dispose();return result==null||result.trim().isEmpty?null:result.trim();
}

class FolderTree extends StatelessWidget {
  final FileHost files;
  final VoidCallback openFolder;
  const FolderTree({super.key, required this.files, required this.openFolder});
  List<Widget> rows(FileWorkspace w, String path, int depth) {
    final page = w.tree[path];
    if (page == null) return [];
    return [
      for (final entry in page.entries) ...[
        ListTile(
          dense: true,
          contentPadding: EdgeInsets.only(left: 8 + depth * 12, right: 4),
          leading: Icon(
            entry['directory'] == true
                ? Icons.folder_outlined
                : Icons.description_outlined,
            size: 16,
          ),
          title: Text(
            entry['name'] as String,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
          onTap: () => entry['directory'] == true
              ? files.expand(w, entry['path'] as String)
              : files.open(w, entry['path'] as String),
        ),
        if (entry['directory'] == true && w.expanded.contains(entry['path']))
          ...rows(w, entry['path'] as String, depth + 1),
      ],
      if (page.error != null)
        Padding(padding: const EdgeInsets.all(8), child: Text(page.error!)),
      if (page.pending) const LinearProgressIndicator(),
      if (page.cursor != null)
        TextButton(
          onPressed: page.pending ? null : () => files.load(w, path),
          child: Text(page.error == null ? 'Load more' : 'Retry'),
        ),
    ];
  }

  @override
  Widget build(BuildContext context) => AnimatedBuilder(
    animation: files,
    builder: (context, _) {
      final w = files.selected;
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                const Expanded(
                  child: Text('Folders', style: TextStyle(fontSize: 18)),
                ),
              if (w != null)
                IconButton(tooltip:'New file',onPressed:() async {
                  final path=await filePathDialog(context,'Create file');if(path==null)return;
                  await files.open(w,path,action:'create');await files.load(w,'.',refresh:true);
                },icon:const Icon(Icons.note_add_outlined,size:18)),
              if (w != null)
                  IconButton(
                    tooltip: 'Refresh files',
                    onPressed: () => files.load(w, '.', refresh: true),
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
              child: ListView(
                children: [
                  for (final path in w.recovery)
                    ListTile(
                      title: Text('Recover $path'),
                      leading: const Icon(Icons.restore),
                      onTap: () => files.open(w, path, action: 'recover'),
                    ),
                  ...rows(w, '.', 0),
                ],
              ),
            ),
        ],
      );
    },
  );
}

class FolderPreview extends StatelessWidget {
  final FileHost files;
  const FolderPreview({super.key, required this.files});
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
                  PopupMenuButton<String>(tooltip:'File actions',onSelected:(action) async {
                    final w=files.selected!;
                    if(action=='rename'){
                      final path=await filePathDialog(context,'Rename file',initial:d.path);if(path!=null)await files.action(w,d,'rename',path:path);
                    }else{
                      final confirmed=await showDialog<bool>(context:context,builder:(context)=>AlertDialog(title:Text('Delete ${d.path}?'),content:const Text('This deletes the saved file from this project. Dirty documents must be saved or closed first.'),actions:[TextButton(onPressed:()=>Navigator.pop(context,false),child:const Text('Cancel')),TextButton(onPressed:()=>Navigator.pop(context,true),child:const Text('Delete'))]));
                      if(confirmed==true)await files.action(w,d,'delete');
                    }
                  },itemBuilder:(_)=>const[PopupMenuItem(value:'rename',child:Text('Rename')),PopupMenuItem(value:'delete',child:Text('Delete'))]),
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
                : SingleChildScrollView(
                    padding: const EdgeInsets.all(16),
                    child: SelectableText(
                      d.text,
                      style: const TextStyle(fontFamily: 'monospace'),
                    ),
                  ),
          ),
        ],
      );
    },
  );
}
