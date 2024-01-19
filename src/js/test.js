/**
 * Test Runner APIs
 *
 * The test module enables the creation of JavaScript tests, drawing
 * inspiration from Deno's built-in test runner.
 *
 * @see {@link https://deno.land/manual/basics/testing}
 *
 * @module Test-Runner
 */

import fs from 'fs';
import { performance } from 'perf_hooks';
import { bg_green, bg_red, red, green, bold } from 'colors';

// Output labels.
const OK = bg_green(bold(' OK '));
const FAIL = bg_red(bold(' FAIL '));

// Regex to match test files.
const TEST_FILE = new RegExp(/.*.spec.ts$|.*.test.ts$|.*.spec.js$|.*.test.js$/);

// Error type referring to test duration timeout.
export class TimeoutError extends Error {
  constructor(message) {
    super();
    this.name = 'TimeoutError';
    this.message = message;
  }
}

// Utility function that wraps a promise with a timeout.
function timeout(promise, time = 0) {
  // When the time is 0ms it means that we don't want to
  // have a timeout for the provided promise.
  if (time === 0) return promise;

  let timerId;
  const timeoutPromise = new Promise((_, reject) => {
    timerId = setTimeout(() => {
      reject(new TimeoutError('Test timed out!'));
    }, time);
  });

  return Promise.race([promise, timeoutPromise]).finally(() => {
    clearTimeout(timerId);
  });
}

// Utility function to join paths similar to Node.js.
function joinPaths(...parts) {
  const separator = '/';
  const replace = new RegExp(separator + '{1,}', 'g');
  return parts.join(separator).replace(replace, separator);
}

/**
 *  TestRunner is the main executor to run JavaScript tests.
 */
export class TestRunner {
  // Initializes the test runner.
  constructor() {
    this.tests = new Map();
    this.testFiles = [];
    this.filter = undefined;
    this.failFast = false;
    this.counters = {
      ok: 0,
      failed: 0,
      ignored: 0,
    };
  }

  /**
   * Registers a new test to the runner.
   *
   * @param {String} description - A brief description of the test.
   * @param {Function} testFn - The test function where the actual test logic is implemented.
   */
  test(description, testFn) {
    // We don't allow tests with similar descriptions.
    if (this.tests.has(description)) {
      throw new Error("Tests can't share the same description.");
    }

    this.tests.set(description, testFn);
  }

  #walkDirs(path, files = []) {
    // Read all files/folders from current path.
    const entities = fs.readdirSync(path);

    for (const filename of entities) {
      const filePath = joinPaths(path, filename);
      const stat = fs.statSync(filePath);

      // Test file has been found.
      if (stat.isFile && filePath.match(TEST_FILE)) {
        files.push(filePath);
        continue;
      }

      // Continue traversing the sub-directories.
      if (stat.isDirectory) {
        this.#walkDirs(filePath, files);
      }
    }
  }

  /**
   * Loads tests from files to the runner recursively.
   *
   * @param {String} [entryPoint] - The path that serves as the starting point for loading tests.
   */
  async importTests(entryPoint = process.cwd()) {
    // Find if the `entryPoint` is file or directory.
    const stat = fs.statSync(entryPoint);

    if (stat.isDirectory) {
      this.#walkDirs(entryPoint, this.testFiles);
    }

    if (stat.isFile) {
      this.testFiles.push(entryPoint);
    }

    await Promise.all(this.testFiles.map((filename) => import(filename)));
  }

  /**
   * Runs all the registered tests as a test suite.
   */
  async run() {
    // Start test suite clock.
    const startTime = performance.now();

    // Run test suite.
    for await (const [description, testFn] of this.tests) {
      // Filter tests based on provided regex.
      if (this.filter && !this.filter.test(description)) {
        continue;
      }

      // Check if the test should be ignored.
      if (testFn.ignore) {
        this.counters.ignored++;
        continue;
      }

      try {
        await timeout(testFn(), testFn.timeout);
        this.counters.ok++;
        console.log(`${OK} ${green(description)}`);
      } catch (err) {
        this.counters.failed++;
        console.log(`${FAIL} ${red(description)}\n ${red(err.stack)}`);

        // Stop running test suite.
        if (this.failFast) {
          const { ok, ignored } = this.counters;
          const remaining = this.tests.size - ok - ignored - 1;
          this.counters.ignored += remaining;
          break;
        }
      }
    }

    const { ok, failed, ignored } = this.counters;

    // Create output strings.
    const elapsedTime = Math.trunc(performance.now() - startTime);
    const result = `${ok} ok; ${failed} failed; ${ignored} ignored`;

    console.log(`\nTest result: ${result} (${elapsedTime} ms)`);

    // Exit with non-zero code if we have test failure.
    process.exit(failed > 0 ? 1 : 0);
  }
}

export const mainRunner = new TestRunner();

function parseOptionsArgs(args) {
  // Check if enough arguments are specified.
  if (args.length < 2) {
    throw new Error(`Not enough arguments specified.`);
  }
  // Use param overloading.
  const defaultOptions = { ignore: false, timeout: 10000 };
  if (typeof args[1] === 'object') {
    args[1] = { ...defaultOptions, ...args[1] };
    return [args[0], args[2], args[1]];
  }
  return [...args, defaultOptions];
}

/**
 * Specifies a test to be registered with the default test runner.
 *
 * @param {string} description - A brief description of the test.
 * @param {string} testFn - The test function where the actual test logic is implemented.
 * @param {Object} [options] - Additional configuration options for the test.
 * @param {boolean} [options.ignore] - The test will be registered but not executed.
 */
function test(...params) {
  // Parse variadic parameters.
  const [description, testFn, options] = parseOptionsArgs(params);

  if (typeof description !== 'string') {
    throw new TypeError(`The "description" argument must be of type string.`);
  }

  if (typeof testFn !== 'function') {
    throw new TypeError(`The "testFn" argument must be of type function.`);
  }

  // Hack: attach options to the test function.
  Object.assign(testFn, options);

  mainRunner.test(description, testFn);
}

export default test;
