import { spawn, spawnSync } from 'node:child_process';
import path from 'node:path';

// tauri-driver bridges WebdriverIO to the platform webview driver.
let tauriDriver;
const binary = path.resolve('../src-tauri/target/release/research-hub');

export const config = {
  hostname: '127.0.0.1',
  port: 4444,
  specs: ['./tests/e2e/**/*.js'],
  maxInstances: 1,
  capabilities: [{ 'tauri:options': { application: binary } }],
  reporters: ['spec'],
  framework: 'mocha',
  mochaOpts: { timeout: 60000 },

  // A stale build would silently test yesterday's code.
  onPrepare: () => spawnSync('cargo', ['tauri', 'build', '--no-bundle'], {
    cwd: '../src-tauri', stdio: 'inherit',
  }),
  beforeSession: () => {
    tauriDriver = spawn('tauri-driver', [], { stdio: [null, process.stdout, process.stderr] });
  },
  afterSession: () => tauriDriver?.kill(),
};
