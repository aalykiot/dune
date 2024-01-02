import fs from 'fs';
import { pipeline } from 'stream';

const decoder = new TextDecoder();

function parseLine(value) {
  const str = decoder.decode(value);
  const items = str.trim().split(',');
  return items.map((s) => s.trim().replaceAll('"', ''));
}

function castToType(value) {
  return isNaN(value) ? String(value) : Number(value);
}

function zipKeyValueLists(values, keys) {
  return values.reduce((item, value, index) => {
    item[keys[index]] = castToType(value.trim());
    return item;
  }, {});
}

async function* splitLines(iterator) {
  let position;
  let buffer = new Uint8Array([]);

  for await (const chunk of iterator) {
    buffer = new Uint8Array([...buffer, ...chunk]);
    position = buffer.indexOf(10);

    while (position >= 0 && buffer.length) {
      yield buffer.subarray(0, position);
      buffer = buffer.subarray(position + 1);
      position = buffer.indexOf(10);
    }
  }
}

async function* csv(iterator) {
  let keys = null;
  for await (const line of iterator) {
    const values = parseLine(line);
    if (!keys) {
      keys = values;
      continue;
    }
    yield zipKeyValueLists(values, keys);
  }
}

async function* toJSON(iterator) {
  for await (const item of iterator) {
    yield `${JSON.stringify(item)}\n`;
  }
}

await pipeline(
  fs.createReadStream('./cities.csv'),
  splitLines,
  csv,
  toJSON,
  process.stdout
);
