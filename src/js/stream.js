// Stream API
//
// A stream is an abstract interface for working with streaming data, and
// is based on generators and async-iterators.
//
// https://youtu.be/YVdw1MDHVZs

import assert from 'assert';
import { EventEmitter } from 'events';

const isFunction = (value) => typeof value === 'function';
const isWritableStream = (value) => value?.write && value?.end;

const pipeStream = (iterator, stream, signal) => async () => {
  // Handle pipeline errors.
  signal.on('uncaughtStreamException', () => stream.end());

  // Consume the async iterator.
  for await (const chunk of iterator) {
    stream.write(chunk);
  }
  stream.end();
};

const pipeDirect = (signal) => (source, target) => {
  // Check if the source is an async iterable.
  assert.true(
    isFunction(source[Symbol.asyncIterator]),
    'Source should be an async iterable.'
  );

  // Pattern match against target's type.
  if (isWritableStream(target)) return pipeStream(source, target, signal);
  if (isFunction(target)) return target(source, signal);

  throw new Error('Unrecognized target type.');
};

const wrap = (iterable, signal) => {
  // Wrap the async iterable object into a readable stream.
  const source = iterable[Symbol.asyncIterator](signal);
  const readable = async function* () {
    yield* source;
  };

  return readable();
};

/**
 * A module method to pipe between streams forwarding errors and properly cleaning up.
 *
 * @param {AsyncGeneratorFunction|AsyncIterator} source
 * @param  {...AsyncGeneratorFunction} targets
 * @returns Promise<undefined>
 */
export function pipeline(source, ...targets) {
  // The signal EE is used to signal the pipeline that an uncaught
  // exception has been thrown and the pipeline is broken.
  const signal = new EventEmitter();
  const sourceWrap = isFunction(source) ? source(signal) : wrap(source, signal);

  const stream = targets.reduce(pipeDirect(signal), sourceWrap);

  // Ensure that the pipeline is a closed circuit.
  if (!isFunction(stream)) {
    throw new Error('The last stream in the pipeline should be a writable.');
  }

  return stream().catch((err) => {
    signal.emit('uncaughtStreamException', err);
    throw err;
  });
}

/**
 * Combines two or more streams into a Duplex stream.
 *
 * @param  {...AsyncGeneratorFunction} targets
 * @returns AsyncGeneratorFunction
 */
export function compose(...targets) {
  // Ensure that the compose stream is an open circuit.
  const last = targets.length - 1;
  if (isWritableStream(targets[last])) {
    throw new Error(`The last stream should be an async generator function.`);
  }

  return function* composeGen(iterator, signal) {
    const stream = targets.reduce(pipeDirect(signal), iterator);
    yield* stream;
  };
}

/**
 * An alias of `pipeline()`.
 */
export const pipe = pipeline;

export default { pipeline, compose, pipe };
