// Public, bounded diagnostic only. No provider credentials or private inputs.
const fs = require('node:fs');
const net = require('node:net');
const path = require('node:path');
const [work, outside, address, packageFile, stdio] = process.argv.slice(process.argv[1]?.endsWith('.cjs') ? 2 : 1);
const details = {};
function attempt(label, operation) {
  try { operation(); return 0; }
  catch (error) {
    details[label] = {code: error.code, errno: error.errno, syscall: error.syscall};
    return error.code || 'UNKNOWN';
  }
}
const report = {
  allowedRead: attempt('allowedRead', () => fs.readFileSync(path.join(work, 'input.txt'))) === 0,
  allowedWrite: attempt('allowedWrite', () => fs.writeFileSync(path.join(work, 'output.txt'), 'public-write')) === 0,
  outsideReadError: attempt('outsideRead', () => fs.readFileSync(outside)),
  outsideWriteError: attempt('outsideWrite', () => fs.writeFileSync(outside, 'changed')),
  packageReadError: attempt('packageRead', () => fs.readFileSync(packageFile)),
  networkAttemptCompleted: false, networkError: null,
};
if (stdio === 'stdio') {
  report.nodeStdio = {
    input: fs.readFileSync(0, 'utf8'),
    stdoutBytes: fs.writeSync(1, 'PUBLIC_STDOUT 世界\n'),
    stderrBytes: fs.writeSync(2, 'PUBLIC_STDERR 世界\n'),
  };
}
let settled = false, socket;
const timer = setTimeout(() => finish('TIMEOUT'), 1500);
function finish(error) {
  if (settled) return;
  settled = true;
  clearTimeout(timer);
  socket?.destroy();
  report.networkAttemptCompleted = error !== 'TIMEOUT';
  report.networkError = error;
  report.nodeErrorDetails = details;
  fs.writeFileSync(path.join(work, 'receipt.json'), JSON.stringify(report));
}
try {
  const split = address.lastIndexOf(':');
  socket = net.createConnection({host: address.slice(0, split), port: Number(address.slice(split + 1))});
  socket.once('connect', () => finish(0));
  socket.once('error', error => {
    details.network = {code: error.code, errno: error.errno, syscall: error.syscall};
    finish(error.code || 'UNKNOWN');
  });
} catch (error) {
  details.networkSetup = {code: error.code, errno: error.errno, syscall: error.syscall};
  // Setup errors are retained, never treated as a denied connect.
  finish('SETUP_FAILED');
}
