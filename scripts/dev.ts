import { spawn } from 'node:child_process';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const [action, ...args] = process.argv.slice(2);
const command = action === 'web' ? ['bunx', 'vite', '--config', 'vite.config.ts', ...args] : null;
if (!command) {
  console.error('Usage: bun scripts/dev.ts web [options]');
  process.exitCode = 2;
} else {
  const child = spawn(command[0], command.slice(1), { cwd: root, stdio: 'inherit' });
  child.on('error', error => { console.error(error); process.exitCode = 1; });
  child.on('exit', (code, signal) => { if (signal) process.kill(process.pid, signal); else process.exitCode = code ?? 1; });
}
