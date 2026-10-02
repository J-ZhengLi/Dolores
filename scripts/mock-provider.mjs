// Local validation fixture; never a production model provider or app sidecar.
import { createServer } from 'node:http';
import { setTimeout } from 'node:timers/promises';

const server = createServer(async (request, response) => {
  if (request.method === 'GET' && request.url === '/v1/models') {
    if (request.headers.authorization && request.headers.authorization !== 'Bearer dolores-generated-restart-test') {
      response.writeHead(401).end(); return;
    }
    response.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({
      object: 'list', data: [{ id: 'dolores-mock', object: 'model' }, { id: 'dolores-fast', object: 'model' }],
    })); return;
  }
  if (request.method !== 'POST' || request.url !== '/v1/chat/completions') {
    response.writeHead(404).end(); return;
  }
  let body = '';
  for await (const chunk of request) {
    body += chunk;
    if (Buffer.byteLength(body) > 256 * 1024) { response.writeHead(413).end(); return; }
  }
  let payload;
  try { payload = JSON.parse(body); }
  catch { response.writeHead(400).end(); return; }
  const input = payload.messages?.at(-1)?.content ?? '';
  if (Array.isArray(payload.tools)) {
    const last = payload.messages?.at(-1);
    const prompt = payload.messages?.findLast(message => message.role === 'user')?.content ?? '';
    if (payload.tools.map(tool => tool?.function?.name).sort().join(',') !== 'edit_text_file,list_folder,read_text_file,search_text') {
      response.writeHead(400).end(); return;
    }
    let message, finish;
    if (!prompt.startsWith('tool-')) {
      message = {role:'assistant',content:'Hello from your working folder. The folder tools are available.'}; finish='stop';
    } else if (prompt.startsWith('tool-edit') && last?.role !== 'tool') {
      message = { role:'assistant', content:null, tool_calls:[{id:'edit-one',type:'function',function:{name:'edit_text_file',arguments:JSON.stringify({path:'readme.txt',old_text:'Hello from an approved workspace file.',new_text:'Updated with an approved edit.'})}}] }; finish='tool_calls';
    } else if (prompt.startsWith('tool-discovery') || prompt.startsWith('tool-search') || prompt === 'tool-list-deny') {
      const number = payload.messages.filter(message => message.role === 'tool').length;
      const previous = payload.messages.at(-2);
      if (last?.role === 'tool' && previous?.tool_calls?.at(-1)?.id !== last.tool_call_id) { response.writeHead(400).end(); return; }
      if (last?.role === 'tool' && last.content.includes('User denied')) {
        message = { role: 'assistant', content: 'The folder operation was denied. No additional file contents were used.' }; finish = 'stop';
      } else if (number === 3) {
        message = { role: 'assistant', content: `Discovered and read the approved note: ${last.content}` }; finish = 'stop';
      } else {
        let name, args;
        if (number === 0) { name = 'list_folder'; args = { path: '.' }; }
        else if (number === 1) {
          const listing = JSON.parse(last.content);
          if (!listing.entries.some(entry => entry.path === 'docs' && entry.kind === 'folder')) { response.writeHead(400).end(); return; }
          name = 'search_text'; args = { path: 'docs', query: '世界.*' };
        } else {
          const search = JSON.parse(last.content);
          if (search.matches?.[0]?.path !== 'docs/notes.txt') { response.writeHead(400).end(); return; }
          name = 'read_text_file'; args = { path: search.matches[0].path };
        }
        message = { role: 'assistant', content: null, tool_calls: [{ id: `discovery-${number}`, type: 'function', function: { name, arguments: JSON.stringify(args) } }] }; finish = 'tool_calls';
      }
    } else if (last?.role === 'tool' && prompt !== 'tool-loop') {
      const previous = payload.messages.at(-2);
      if (previous?.tool_calls?.at(-1)?.id !== last.tool_call_id) { response.writeHead(400).end(); return; }
      message = { role: 'assistant', content: last.content.includes('User denied')
        ? 'The file read was denied. No file contents were used.'
        : `The approved file says: ${last.content}` };
      finish = 'stop';
    } else {
      const number = payload.messages.filter(message => message.role === 'tool').length;
      const path = prompt === 'tool-escape' ? '../outside.txt' : 'readme.txt';
      message = { role: 'assistant', content: null, tool_calls: [{ id: `file-${number}`, type: 'function', function: { name: 'read_text_file', arguments: JSON.stringify({ path }) } }] };
      finish = 'tool_calls';
    }
    if (payload.stream === true) {
      response.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' });
      let closed = false;
      response.on('close', () => { closed = true; });
      const send = (delta, finish = null) => response.write(`data: ${JSON.stringify({choices:[{index:0,delta,finish_reason:finish}]})}\n\n`);
      if (message.tool_calls) {
        for (const part of 'I’ll inspect the requested folder or file, with your approval.'.match(/.{1,12}/gs)) {
          if (closed) return;
          send({content:part});
          await setTimeout(15);
        }
        const call = message.tool_calls[0];
        send({tool_calls:[{index:0,id:call.id,type:'function',function:{name:call.function.name,arguments:''}}]});
        for (const part of call.function.arguments.match(/.{1,8}/gs)) {
          if (closed) return;
          send({tool_calls:[{index:0,function:{arguments:part}}]});
          await setTimeout(prompt === 'tool-stream-slow' ? 500 : 15);
          if (prompt === 'tool-stream-incomplete') { response.end(); return; }
        }
      } else {
        for (const part of message.content.match(/.{1,12}/gs) ?? []) {
          if (closed) return;
          send({content:part});
          await setTimeout(25);
        }
      }
      send({}, finish);
      if (payload.stream_options?.include_usage) {
        response.write(`data: ${JSON.stringify({choices:[],usage:{prompt_tokens:40,completion_tokens:12,total_tokens:52}})}\n\n`);
      }
      response.end('data: [DONE]\n\n');
      return;
    }
    response.writeHead(200, { 'content-type': 'application/json' }).end(JSON.stringify({
      choices: [{message, finish_reason:finish}], usage:{prompt_tokens:40, completion_tokens:12, total_tokens:52},
    })); return;
  }
  if (input === 'limit-check' && payload.max_tokens !== 4096) { response.writeHead(400).end(); return; }
  if (input === 'model-check' && payload.model !== 'dolores-fast') { response.writeHead(400).end(); return; }
  if (input === 'credential-check' && request.headers.authorization !== 'Bearer dolores-generated-restart-test') {
    response.writeHead(401).end(); return;
  }
  if (input === 'fail') { response.writeHead(401, { 'content-type': 'application/json' }).end(JSON.stringify({ error: { message: 'fixture-private-error-body' } })); return; }
  response.writeHead(200, { 'content-type': 'text/event-stream', 'cache-control': 'no-cache' });
  let closed = false;
  response.on('close', () => { closed = true; });
  const text = input === 'markdown'
    ? '# A small Rust example\n\n**Readable replies**, `inline code`, and Unicode: 你好.\n\n- Stream the answer\n- Keep the source\n\n| Feature | Result |\n| --- | --- |\n| Markdown | Native widgets |\n| Code | Select and copy |\n\n```rust\nfn main() {\n    println!("Hello, Dolores — 你好!"); // a long code line scrolls horizontally inside its own block without widening the conversation\n}\n```\n\nRICH_END'
    : input === 'slow'
    ? 'This is a deliberately slow response from the local test server. Stop it to check cancellation.'
    : 'Hello from the Dolores local test server. 你好！\n\nStreaming, conversation storage, and your desktop connection are working. This is a fixture response, not a language model.';
  for (const word of text.match(/.{1,12}/gs) ?? []) {
    if (closed) return;
    response.write(`data: ${JSON.stringify({ choices: [{ delta: { content: word }, finish_reason: null }] })}\n\n`);
    await setTimeout(input === 'slow' ? 500 : input === 'markdown' ? 75 : 25);
    if (input === 'truncated') { response.end(); return; }
  }
  response.write('data: {"choices":[{"delta":{},"finish_reason":"stop"}]}\n\n');
  if (payload.stream_options?.include_usage && input !== 'no-usage') {
    response.write(`data: ${JSON.stringify({ choices: [], usage: { prompt_tokens: 64, completion_tokens: 32, total_tokens: 96, prompt_tokens_details: { cached_tokens: 0 } } })}\n\n`);
  }
  response.end('data: [DONE]\n\n');
});

server.listen(19421, '127.0.0.1', () => {
  console.log('Dolores test endpoint: http://127.0.0.1:19421/v1 — model: dolores-mock');
  console.log('Send "slow" to test stop, "fail" for denial, "truncated" for interruption.');
});
