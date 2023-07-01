import test from 'test';
import assert from 'assert';

function doSomeMath(a, b) {
  return a + b;
}

test('checking multiple addition values (1)', () => {
  for (let a = 1; a < 10; a++) {
    assert.equal(doSomeMath(a, 5), a + 5);
  }
});

test('checking multiple addition values (2)', () => {
  for (let a = 1; a < 10; a++) {
    assert.equal(doSomeMath(a, 5), a + 6);
  }
});

test('checking multiple addition values (3)', () => {
  for (let a = 1; a < 10; a++) {
    assert.equal(doSomeMath(a, 5), a + 5);
  }
});

test('checking multiple addition values (4)', () => {
  for (let a = 1; a < 10; a++) {
    assert.equal(doSomeMath(a, 5), a + 5);
  }
});

test('checking multiple addition values (5)', { ignore: true }, () => {
  for (let a = 1; a < 10; a++) {
    assert.equal(doSomeMath(a, 5), a + 5);
  }
});

test('checking multiple addition values (6)', () => {
  for (let a = 1; a < 10; a++) {
    assert.equal(doSomeMath(a, 5), a + 5);
  }
});
