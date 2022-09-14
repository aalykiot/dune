import timers from 'timers';
import { Console } from 'console';
import { TextEncoder, TextDecoder } from 'text-encoding';
import { cloneFunction, parseEnvVariable } from 'util';
import { readFileSync } from 'fs';

globalThis.global = globalThis;

global.GLOBAL = globalThis;
global.root = globalThis;

function makeGlobal(name, value) {
  globalThis[name] = value;
}

// Note: Adding a caching layer to `process.binding` allows us to not
// cross the JavaScript <-> Rust bridge every time we need native methods.

let cache = new Map();
let internalBinding = cloneFunction(process.binding);

process.binding = (name) => {
  // Check bindings cache.
  if (cache.has(name)) return cache.get(name);

  // Load binding (from rust), save it to cache.
  const binding = internalBinding(name);
  cache.set(name, binding);

  return binding;
};

const kill = cloneFunction(process.kill);

process.kill = (pid, signal = 'SIGKILL') => {
  // Check arguments.
  if (!pid || Number.isNaN(Number.parseInt(pid))) {
    throw new TypeError(`The "pid" argument must be of type number.`);
  }
  kill(pid, signal);
};

// Setting up the STDOUT stream.

let stdout = process.stdout;

Object.defineProperty(process, 'stdout', {
  get() {
    // Don't initialize twice.
    if (stdout) return stdout;

    // Setup stdout stream.
    const binding = process.binding('stdio');

    return {
      write(value) {
        binding.write(value);
      },
    };
  },
  configurable: true,
});

// Setting up the STDERR stream.

let stderr = process.stderr;

Object.defineProperty(process, 'stderr', {
  get() {
    // Don't initialize twice.
    if (stderr) return stderr;

    // Setup stderr stream.
    const binding = process.binding('stdio');

    return {
      write(value) {
        binding.writeError(value);
      },
    };
  },
  configurable: true,
});

makeGlobal('console', new Console());

makeGlobal('setTimeout', timers.setTimeout);
makeGlobal('setInterval', timers.setInterval);
makeGlobal('setImmediate', timers.setImmediate);
makeGlobal('clearTimeout', timers.clearTimeout);
makeGlobal('clearInterval', timers.clearInterval);
makeGlobal('clearImmediate', timers.clearImmediate);

makeGlobal('TextEncoder', TextEncoder);
makeGlobal('TextDecoder', TextDecoder);

// Loading env variables from a .env file automatically.

const DOTENV_FILE = process.cwd() + '/.env';
const DOTENV_COMMENTS = /(?<=^([^"']|"[^"']*")*)#.*/g;

try {
  const dotenvContent = readFileSync(DOTENV_FILE, 'utf-8');
  const dotenv = dotenvContent
    .split('\n')
    .map((env) => env.replace(DOTENV_COMMENTS, '').trim())
    .filter((env) => env !== '');

  dotenv.forEach((env) => {
    const [key, value] = env.split('=');
    process.env[key.trim()] = parseEnvVariable(value);
  });
} catch (_) {
  // We don't care about handling the error.
}
