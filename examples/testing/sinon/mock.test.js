// Example copied from: https://deno.land/manual/basics/testing#example-spying-on-a-function-with-sinon

import sinon from 'https://cdn.skypack.dev/sinon';
import test from 'test';
import assert from 'assert';
import { bar, foo } from './myFile';

test('calls bar during execution of foo', () => {
  // Create a test spy that wraps 'bar'
  const spy = sinon.spy(bar);

  // Call function 'foo' and pass the spy as an argument
  foo(spy);

  assert.equal(spy.called, true);
  assert.equal(spy.getCalls().length, 1);
});
