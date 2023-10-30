import test from 'test';
import assert from 'assert';
import fs from 'fs';
import { pipeline } from 'stream';

test('[FILE-SYSTEM] Reads current test file into a Uint8Array.', async () => {
  const content = await fs.readFile(import.meta.url);
  assert.true(content instanceof Uint8Array);
});

test('[FILE-SYSTEM] Reads current test file as a string.', async () => {
  const content = await fs.readFile(import.meta.url, { encoding: 'utf-8' });
  assert.true(typeof content === 'string');
});

test('[FILE-SYSTEM] Reads current test file as stream.', async () => {
  let content = '';
  const stream = fs.createReadStream(import.meta.url, { encoding: 'utf-8' });
  const sink = { write: (data) => (content += data), end: () => {} };
  await pipeline(stream, sink);
  assert.true(content.startsWith(`import test from 'test';`));
});

test('[FILE-SYSTEM] Writes data into a temp file.', async () => {
  const tempFile = 'tmp_01.txt';
  const data = 'Welcome to Dune 🪐';
  await fs.writeFile(tempFile, data);
  const content = await fs.readFile(tempFile, { encoding: 'utf-8' });
  await fs.rm(tempFile);
  assert.equal(content, data);
});

test('[FILE-SYSTEM] Crates a directory in current path.', async () => {
  const tempDir = `./tmp_${process.pid}`;
  await fs.mkdir(tempDir);
  const stat = await fs.stat(tempDir);
  await fs.rm(tempDir);
  assert.true(stat.isDirectory);
});
