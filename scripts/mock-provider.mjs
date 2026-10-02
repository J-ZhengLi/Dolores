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
