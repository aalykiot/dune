import test from 'test';
import assert from 'assert';

const options = { timeout: 500 };

test('[TIMERS] Promise microtask should be supported.', options, () => {
  Promise.resolve();
  assert.true(true);
});

test('[TIMERS] SetTimeout should be supported.', options, async () => {
  await new Promise((resolve) => {
    setTimeout(resolve, 100);
  });
  assert.true(true);
});

test('[TIMERS] SetTimeout should accept params.', options, async () => {
  await new Promise((resolve) => {
    const cb = (arg1, arg2) => {
      assert.equal(arg1, 'A');
      assert.equal(arg2, 'B');
      resolve();
    };
    setTimeout(cb, 100, 'A', 'B');
  });
  assert.true(true);
});

test('[TIMERS] SetInterval should be supported.', options, async () => {
  let count = 0;
  await new Promise((resolve) => {
    const id = setInterval(() => {
      if (++count === 3) {
        clearImmediate(id);
        resolve();
      }
    }, 50);
  });
  assert.equal(count, 3);
});

test('[TIMERS] SetImmediate should be supported.', options, async () => {
  await new Promise((resolve) => {
    setImmediate(resolve);
  });
  assert.true(true);
});

test('[TIMERS] ClearTimeout should be supported.', options, () => {
  let count = 0;
  const id = setTimeout(() => count++, 100);
  clearTimeout(id);
  assert.equal(count, 0);
});

test('[TIMERS] ClearImmediate should be supported.', options, () => {
  let data = 0;
  const id = setImmediate(() => data++);
  clearImmediate(id);
  assert.equal(data, 0);
});
