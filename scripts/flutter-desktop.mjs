// Development helper only; Node is never part of the shipped app.
import { spawnSync, spawn } from 'node:child_process';
import { copyFileSync, existsSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const app = resolve(root, 'apps/dolores_flutter');
const localSdk = resolve(root, 'output/toolchains/flutter');
const sdk = process.env.FLUTTER_SDK || (existsSync(localSdk) ? localSdk : null);
const platform = { win32: 'windows', linux: 'linux', darwin: 'macos' }[process.platform];
if (!platform) throw new Error('Desktop targets are Windows, macOS and Linux.');
function run(command, args, cwd = root) {
  const result = spawnSync(command, args, { cwd, stdio: 'inherit', env: {
    ...process.env, FLUTTER_SUPPRESS_ANALYTICS: 'true', DART_SUPPRESS_ANALYTICS: 'true',
  }});
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
let executable;
if (platform === 'windows') {
  // flutter.bat is resolved by PowerShell without shell-string interpolation.
  const args = ['-NoProfile', '-File', resolve(root, 'scripts/build-flutter.ps1')];
  if (sdk) args.push('-FlutterSdk', sdk);
  run('powershell.exe', args);
  executable = resolve(app, 'build/windows/x64/runner/Release/dolores_flutter.exe');
} else {
  const flutter = sdk ? resolve(sdk, 'bin/flutter') : 'flutter';
  run('cargo', ['build', '-p', 'dolores-flutter-bridge', '--release', '--locked']);
  run(flutter, ['--no-version-check', '--suppress-analytics', 'pub', 'get'], app);
  run(flutter, ['--no-version-check', '--suppress-analytics', 'build', platform, '--release'], app);
  if (platform === 'linux') {
    const bundle = resolve(app, 'build/linux/x64/release/bundle');
    copyFileSync(resolve(root, 'target/release/libdolores_flutter_bridge.so'), resolve(bundle, 'lib/libdolores_flutter_bridge.so'));
    executable = resolve(bundle, 'dolores_flutter');
  } else {
    const bundle = resolve(app, 'build/macos/Build/Products/Release/dolores_flutter.app/Contents');
    copyFileSync(resolve(root, 'target/release/libdolores_flutter_bridge.dylib'), resolve(bundle, 'Frameworks/libdolores_flutter_bridge.dylib'));
    executable = resolve(bundle, 'MacOS/dolores_flutter');
  }
}
if (!process.argv.includes('--build-only')) {
  const child = spawn(executable, [], { cwd: dirname(executable), detached: true, stdio: 'ignore' });
  child.on('error', error => { console.error(error.message); process.exitCode = 1; });
  child.unref();
}
