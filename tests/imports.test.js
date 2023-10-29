import test from 'test';
import assert from 'assert';
import _ from 'https://cdn.skypack.dev/lodash';
import data from './fixtures/data.json';
import { num } from './helpers/function';

const options = { timeout: 5000 };

test('[IMPORTS] Deep path imports works.', options, () => {
  assert.equal(num(), 42);
});

test('[IMPORTS] Dynamic imports work.', options, async () => {
  const { num } = await import('./helpers/function');
  assert.equal(num(), 42);
});

test('[IMPORTS] URL imports works.', options, () => {
  const words = ['sky', 'wood', 'forest', 'ocean', 'universe'];
  assert.equal(_.first(words), 'sky');
  assert.equal(_.last(words), 'universe');
});

test('[IMPORTS] JSON imports work.', options, () => {
  assert.equal(data?.fruit, 'Apple');
  assert.equal(data?.size, 'Large');
  assert.equal(data?.color, 'Red');
});

// TODO: Figure out why this test behaves strangely. 🤔
test('[IMPORTS] WASM imports work.', { ignore: true }, async () => {
  const { default: calc } = await import('./helpers/calc.wasm');
  assert.equal(calc.addTwo(2, 3), 5);
});
