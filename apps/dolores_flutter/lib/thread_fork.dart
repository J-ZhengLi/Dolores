import 'package:flutter/material.dart';
import 'chat.dart';
Future<void> showThreadFork(BuildContext context,ChatController chat) async {
 final turns=chat.messages.where((m)=>m['role']=='assistant'&&m['id'] is int).toList();
 final through=await showDialog<int>(context:context,builder:(context)=>AlertDialog(
  title:const Text('Fork conversation'),
  content:SizedBox(width:480,height:320,child:Column(children:[
   const Text('The new chat shares this working folder. Files are shared; active runs, drafts and permissions are not copied. Choose a completed turn. Earlier turns can be loaded from chat history.'),
   Expanded(child:ListView(children:[for(final turn in turns)ListTile(title:Text(turn['content'] as String,maxLines:2,overflow:TextOverflow.ellipsis),onTap:()=>Navigator.pop(context,turn['id'] as int))])),
  ])),actions:[TextButton(onPressed:()=>Navigator.pop(context),child:const Text('Cancel'))]));
 if(through!=null)await chat.forkAt(through);
}
