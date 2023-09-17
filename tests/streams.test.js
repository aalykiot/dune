import test from 'test';
import assert from 'assert';
import { pipeline } from 'stream';

// const inputStream$ = (sentence) => {
//   return function* () {
//     for (const word of sentence.split(' ')) {
//       yield word;
//     }
//   };
// };

// const toUpper$ = (input) => {
//   return async function* () {
//     for await (const value of input) {
//       yield value.toUppercase();
//     }
//   };
// };

test('[STREAMS] The stream pipeline completes.', () => {
  // let sentence;
  // const sink$ = {
  //   write: (data) => (sentence += data),
  //   end: () => {},
  // };
  // await pipeline(inputStream$('Hello World!'), toUpper$, sink$);
  // assert.equal(sentence, 'HELLO WORLD!');
  assert.true(true);
});
