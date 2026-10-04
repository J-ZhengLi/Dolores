const { test } = require('node:test');
const assert = require('node:assert/strict');
const { createServer } = require('node:http');
const { mkdtemp, readFile, rm, writeFile } = require('node:fs/promises');
const os = require('node:os');
const path = require('node:path');
const { chromium } = require('playwright-core');
const { Session, address, modulePath } = require('./worker.cjs');

test('URLs exclude file/credential/plain remote HTTP navigation', () => {
  assert.equal(modulePath('\\\\?\\D:\\synthetic folder'), 'D:\\synthetic folder');
  assert.equal(modulePath('\\\\?\\UNC\\server\\share'), '\\\\server\\share');
  for (const url of ['file:///secret','http://example.org','https://name:secret@example.org']) assert.throws(() => address(url));
  assert.equal(address('http://127.0.0.1:8000').hostname, '127.0.0.1');
});
test('real synthetic page: reversible input, stale state, timeout, capture and closed tab', { timeout: 45000 }, async () => {
  const server = createServer((req, res) => {
    res.setHeader('Content-Type', 'text/html');
    res.end('<title>Dolores synthetic browser test</title><label>Name <input id="name"></label><button onclick="document.querySelector(\'#result\').textContent=document.querySelector(\'#name\').value">Preview</button><p id="result">Waiting</p><input type="password" value="PRIVATE_PASSWORD"><input type="hidden" value="PRIVATE_HIDDEN"><button disabled>Disabled</button><button onclick="document.body.textContent=\'popup blocked\';window.open(\'https://example.org\')">Popup</button>');
  });
  await new Promise(resolve => server.listen(0, '127.0.0.1', resolve));
  const dir = await mkdtemp(path.join(os.tmpdir(), 'dolores-browser-'));
  const s = new Session(chromium, dir, process.platform === 'win32' ? 'msedge' : 'chrome');
  try {
    const opened = await s.run({ operation: 'open', url: `http://127.0.0.1:${server.address().port}/` });
    assert.equal(opened.outcome, 'opened');
    assert.equal(opened.controls.length, 4);
    assert.ok(!JSON.stringify(opened).includes('PRIVATE_'));
    const filled = await s.run({ operation: 'fill', ref: opened.controls[0].ref, state: opened.state, text: 'Synthetic name' });
    assert.equal(filled.outcome, 'acted');
    const stale = await s.run({ operation: 'click', ref: 'e2', state: opened.state });
    assert.equal(stale.outcome, 'stale'); assert.equal(stale.actionDispatched, false);
    assert.ok(stale.text.includes('Waiting'));
    const clicked = await s.run({ operation: 'click', ref: 'e2', state: stale.state });
    assert.equal(clicked.outcome, 'acted'); assert.ok(clicked.text.includes('Synthetic name'));
    const captured = await s.run({ operation: 'screenshot', state: clicked.state });
    const bytes = await readFile(path.join(dir, `${captured.capture}.jpg`));
    assert.ok(bytes.length < 512 * 1024); assert.equal(bytes[0], 255);
    await s.page.evaluate(() => document.querySelector('#result').textContent = 'Changed outside Dolores');
    const changed = await s.run({ operation: 'fill', ref: 'e1', state: captured.state, text: 'Must not run' });
    assert.equal(changed.outcome, 'stale'); assert.ok(!changed.controls[0].value.includes('Must not run'));
    const timeout = await s.run({ operation: 'click', ref: 'e3', state: changed.state });
    assert.equal(timeout.outcome, 'uncertain'); assert.ok(timeout.recovery.includes('nothing was retried'));
    // A previous uncertain token cannot replay even an ordinary control.
    const expired = await s.run({ operation: 'click', ref: 'e2', state: changed.state });
    assert.equal(expired.outcome, 'stale');
    await s.page.close();
    await assert.rejects(s.run({ operation: 'state' }), /closed/);
    const recovered = await s.run({ operation: 'open', url: `http://127.0.0.1:${server.address().port}/` });
    assert.equal(recovered.outcome, 'opened');
    const popup = await s.run({ operation: 'click', ref: 'e4', state: recovered.state });
    assert.ok(popup.text.includes('popup blocked')); assert.equal(s.context.pages().length, 1);
    await s.page.evaluate(() => document.body.textContent = '界'.repeat(4000));
    const unicode = await s.run({ operation: 'state' });
    assert.equal(unicode.partial, true); assert.ok(Buffer.byteLength(unicode.text) <= 8192);
    for (let i = 0; i < 128; i++) await writeFile(path.join(dir, `fixture-${i}.jpg`), 'synthetic');
    const limited = await s.run({ operation: 'screenshot', state: unicode.state });
    assert.equal(limited.outcome, 'limited'); assert.ok(limited.recovery.includes('128 images'));
    assert.ok(limited.text.includes('界'));
    await s.run({ operation: 'close' }); assert.equal(s.browser, null);
    console.log(JSON.stringify({ syntheticNavigationAndForm: true, staleNoReplay: true, timeoutRecovery: true, screenshotBytes: bytes.length, closedTabRecovery: true, ownedCleanup: true }));
  } finally { await s.close(); server.close(); await rm(dir, { recursive: true, force: true }); }
});
