import test from 'test';
import assert from 'assert';
import { pipeline } from 'stream';

async function* inputStream$(sentence) {
  yield* sentence.split(' ');
}

async function* toUpperCase$(source) {
  for await (const value of source) {
    yield value.toUpperCase();
  }
}

test('[STREAMS] The stream pipeline completes.', async () => {
  let sentence = '';
  const sink$ = {
    write: (data) => (sentence += `${data}`),
    end: () => {},
  };
  await pipeline(inputStream$('Hello World!'), toUpperCase$, sink$);
  assert.equal(sentence, 'HELLOWORLD!');
});
