import { echoC } from './c.js';

export function echoB(value) {
  echoC(`B:${value}`);
}
