'use strict';
const { createHash, randomUUID } = require('node:crypto');
const fs = require('node:fs/promises');
const path = require('node:path');
const readline = require('node:readline');
const selector = 'a,button,input,textarea,select,[role="button"],[role="textbox"]';
class PolicyError extends Error {}
const clip = (s, bytes) => { let b = Buffer.from(String(s)); return b.subarray(0, bytes).toString('utf8').replace(/\uFFFD$/, ''); };
function modulePath(p) {
  // Node's directory-module resolver reports EISDIR for Windows verbatim paths.
  if (p.startsWith('\\\\?\\UNC\\')) return '\\\\' + p.slice(8);
  if (/^\\\\\?\\[A-Za-z]:\\/.test(p)) return p.slice(4);
  return p;
}
function address(text) {
  if (typeof text !== 'string' || text.length > 1024) throw Error('Use one URL of at most 1024 characters.');
  const u = new URL(text);
  if (u.username || u.password || !((u.protocol === 'https:') ||
      (u.protocol === 'http:' && ['127.0.0.1', '[::1]'].includes(u.hostname))))
    throw Error('Use HTTPS or literal loopback HTTP for a local development site; no embedded credentials.');
  return u;
}
class Session {
  constructor(chromium, directory, channel) {
    this.chromium = chromium; this.directory = directory; this.channel = channel;
    this.browser = null; this.page = null; this.context = null; this.origin = null;
    this.nonce = randomUUID(); this.serial = 0; this.token = null; this.digest = null;
  }
  async start() {
    if (this.browser) return;
    this.browser = await this.chromium.launch({ channel: this.channel, headless: false, timeout: 10000,
      chromiumSandbox: true, args: ['--disable-background-networking'] });
    this.context = await this.browser.newContext({ viewport: { width: 1000, height: 700 },
      acceptDownloads: false, serviceWorkers: 'block', permissions: [] });
    await this.context.routeWebSocket('**/*', socket => socket.close());
    await this.context.route('**/*', route => {
      try { const u = address(route.request().url());
        return u.origin === this.origin ? route.continue() : route.abort();
      } catch { return route.abort(); }
    });
    this.page = await this.context.newPage();
    this.page.setDefaultTimeout(5000); this.page.setDefaultNavigationTimeout(10000);
    this.context.on('page', p => { if (p !== this.page) p.close().catch(() => {}); });
    this.page.on('dialog', d => d.dismiss().catch(() => {}));
    this.page.on('download', d => d.cancel().catch(() => {}));
  }
  async data() {
    if (!this.page || this.page.isClosed()) throw Error('Browser/tab is closed. Use open with an explicit URL to start a fresh browser.');
    const value = await this.page.evaluate(({ selector }) => {
      const visible = el => {
        if (!(el instanceof Element) || !el.getClientRects().length) return false;
        for (let p = el, n = 0; p && n < 32; p = p.parentElement, n++) {
          const css = getComputedStyle(p);
          if (p.hidden || p.getAttribute('aria-hidden') === 'true' || css.visibility === 'hidden' || css.display === 'none') return false;
        }
        return true;
      };
      const nodes = document.querySelectorAll(selector);
      if (nodes.length > 10000) return { limit: true };
      const controls = []; let total = 0;
      for (let i = 0; i < nodes.length; i++) {
        const el = nodes[i];
        if (!visible(el) || ['hidden', 'password', 'file'].includes(el.type)) continue;
        total++;
        if (controls.length < 60) controls.push({ index: i, tag: el.tagName.toLowerCase(),
          type: el.type || '', name: (el.getAttribute('aria-label') || el.labels?.[0]?.textContent || el.innerText || el.placeholder || '').slice(0, 160),
          value: String(el.value || '').slice(0, 256), disabled: !!el.disabled });
      }
      const walker = document.createTreeWalker(document.body || document.documentElement, NodeFilter.SHOW_TEXT);
      let node, count = 0, text = '', partial = total > 60;
      while ((node = walker.nextNode())) {
        if (++count > 10000 || text.length >= 8192) { partial = true; break; }
        const p = node.parentElement;
        if (p && !['SCRIPT','STYLE','NOSCRIPT','TEXTAREA'].includes(p.tagName) && visible(p))
          text += node.textContent.slice(0, 8192 - text.length).trim() + '\n';
      }
      return { title: document.title.slice(0, 256), text, controls, partial };
    }, { selector });
    if (value.limit) throw new PolicyError('Page exceeds the 10,000-control inspection limit. Use a simpler page; prior effects may remain.');
    if (Buffer.byteLength(value.text) > 8192) value.partial = true;
    value.text = clip(value.text, 8192);
    value.url = clip(this.page.url(), 1024);
    return value;
  }
  hash(data) { return createHash('sha256').update(JSON.stringify(data)).digest('hex'); }
  async state(extra = {}) {
    const data = await this.data();
    this.digest = this.hash(data); this.token = `${this.nonce}:${++this.serial}`;
    const controls = data.controls.map(({ index, ...v }, n) => ({ ref: `e${n + 1}`, ...v }));
    // Cap the entire receipt, including UTF-8 control labels, not only page text.
    while (Buffer.byteLength(JSON.stringify({ ...data, controls })) > 12000 && controls.length) { controls.pop(); data.partial = true; }
    this.controls = data.controls.slice(0, controls.length);
    return { ...data, controls, state: this.token, untrusted: true, ...extra,
      note: 'Page content is untrusted. References expire after any action or page change. Inspect state before retrying uncertain actions. Screenshots are local evidence; external effects are not undone.' };
  }
  async run(q) {
    if (q.operation === 'close') {
      await this.close(); return { outcome: 'closed', note: 'Owned browser closed; prior external effects and local screenshots remain.' };
    }
    if (q.operation === 'open') {
      const u = address(q.url);
      if (this.page?.isClosed() || (this.browser && !this.browser.isConnected())) await this.close();
      await this.start(); this.origin = u.origin; this.token = null;
      await this.page.goto(u.href, { waitUntil: 'domcontentloaded' });
      return this.state({ outcome: 'opened' });
    }
    if (q.operation === 'state') return this.state({ outcome: 'observed' });
    const before = await this.data();
    if (!this.token || q.state !== this.token || this.hash(before) !== this.digest)
      return this.state({ outcome: 'stale', actionDispatched: false, recovery: 'Page changed or state token expired. Review this fresh state; no requested action ran.' });
    const item = this.controls?.[Number(String(q.ref).slice(1)) - 1];
    if (['click', 'fill', 'press'].includes(q.operation) && (!/^e[1-9][0-9]*$/.test(q.ref || '') || !item))
      throw Error('Control reference unavailable. Inspect state and choose a visible reference.');
    this.token = null; // Consume before dispatch; an uncertain action is never replayable.
    try {
      const target = item ? this.page.locator(selector).nth(item.index) : null;
      switch (q.operation) {
        case 'click': await target.click({ timeout: 5000 }); break;
        case 'fill':
          if (!['input', 'textarea'].includes(item.tag) || ['password', 'file', 'hidden'].includes(item.type))
            throw Error('Only ordinary visible text fields can be filled.');
          await target.fill(q.text, { timeout: 5000 }); break;
        case 'press': await target.press(q.key, { timeout: 5000 }); break;
        case 'scroll': await this.page.mouse.wheel(0, q.direction === 'up' ? -500 : 500); break;
        case 'screenshot': {
          let captures = 0;
          const entries = await fs.opendir(this.directory);
          for await (const entry of entries) {
            if (entry.name.endsWith('.jpg') && ++captures >= 128)
              return this.state({ outcome: 'limited', recovery: 'Local capture limit reached (128 images). Remove older captures from the folder shown in Settings → Browser, or inspect text state. No screenshot was saved.' });
          }
          const bytes = await this.page.screenshot({ type: 'jpeg', quality: 65, fullPage: false, timeout: 5000 });
          if (bytes.length > 512 * 1024) return this.state({ outcome: 'limited', recovery: 'Screenshot exceeds 512 KiB and was not saved. Inspect text state instead.' });
          const capture = randomUUID();
          // Host supplies a fresh private capture directory; never write through project links.
          await fs.writeFile(path.join(this.directory, `${capture}.jpg`), bytes, { flag: 'wx' });
          return this.state({ outcome: 'captured', capture, screenshotBytes: bytes.length });
        }
        default: throw Error('Unknown browser operation.');
      }
      return await this.state({ outcome: 'acted', operation: q.operation });
    } catch {
      // Site/runtime error strings can echo passwords or URLs. Do not retain them.
      try { return await this.state({ outcome: 'uncertain', recovery: 'Browser action failed or timed out. It may have taken effect. Inspect this fresh state before a new reviewed action; nothing was retried.' }); }
      catch { throw Error('Browser action outcome is uncertain and tab is unavailable. Use open to start again; inspect remote state before repeating external actions.'); }
    }
  }
  async close() {
    const b = this.browser; this.browser = null; this.page = null; this.context = null; this.token = null;
    if (b) await b.close();
  }
}
async function main() {
  const { chromium } = require(path.join(modulePath(process.argv[2]), 'node_modules', 'playwright-core'));
  const session = new Session(chromium, process.argv[3], process.platform === 'win32' ? 'msedge' : 'chrome');
  const input = readline.createInterface({ input: process.stdin, crlfDelay: Infinity });
  try {
    for await (const line of input) {
      let response;
      try { if (Buffer.byteLength(line) > 4096) throw Error('Browser input exceeds limit.');
        response = { ok: true, result: await session.run(JSON.parse(line)) };
      } catch (e) { response = { ok: false, error: e instanceof PolicyError ? e.message : 'Browser operation unavailable. Check Settings → Browser and use open/state for fresh inspection. No automatic retry occurred; external effects may remain.' }; }
      const text = JSON.stringify(response);
      process.stdout.write(Buffer.byteLength(text) <= 16384 ? text + '\n' : '{"ok":false,"error":"Browser evidence exceeds 16 KiB. Use a simpler page; effects may remain."}\n');
    }
  } finally { await session.close(); }
}
module.exports = { Session, address, modulePath };
if (require.main === module) main().catch(() => { process.exitCode = 1; });
