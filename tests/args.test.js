import test from 'test';
import assert from 'assert';

test('[ARGS] CLI arguments is an array.', () => {
  assert.true(Array.isArray(process.argv));
});
