import { echoB } from './b.js';

export function echoA(value) {
  echoB(`A:${value}`);
}
