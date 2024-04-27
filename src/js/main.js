import process from 'process';
import timers from 'timers';
import fetch from '@web/fetch';
import structuredClone from '@web/clone';
import { Console, prompt, wrapConsole } from 'console';
import { AbortController, AbortSignal } from '@web/abort';
import { TextEncoder, TextDecoder } from '@web/text_encoding';

globalThis.global = globalThis;

global.GLOBAL = globalThis;
global.root = globalThis;

function makeGlobal(name, value) {
  globalThis[name] = value;
}

const { $$queueMicro, reportError } = globalThis;

// Note: We wrap `queueMicrotask` and manually emit the exception because
// v8 doesn't provide any mechanism to handle callback exceptions during
// the microtask_checkpoint phase.
function queueMicrotask(callback) {
  // Check if the callback argument is a valid type.
  if (typeof callback !== 'function') {
    throw new TypeError(`The "callback" argument must be of type function.`);
  }

  $$queueMicro(() => {
    try {
      callback();
    } catch (err) {
      reportError(err);
    }
  });
}

const console = new Console();
const consoleFromV8 = globalThis['console'];

wrapConsole(console, consoleFromV8);

/* Initialize global environment for user script */

makeGlobal('process', process);
makeGlobal('queueMicrotask', queueMicrotask);
makeGlobal('console', console);
makeGlobal('prompt', prompt);

makeGlobal('setTimeout', timers.setTimeout);
makeGlobal('setInterval', timers.setInterval);
makeGlobal('setImmediate', timers.setImmediate);
makeGlobal('clearTimeout', timers.clearTimeout);
makeGlobal('clearInterval', timers.clearInterval);
makeGlobal('clearImmediate', timers.clearImmediate);

makeGlobal('TextEncoder', TextEncoder);
makeGlobal('TextDecoder', TextDecoder);
makeGlobal('structuredClone', structuredClone);
makeGlobal('AbortController', AbortController);
makeGlobal('AbortSignal', AbortSignal);
makeGlobal('fetch', fetch);
