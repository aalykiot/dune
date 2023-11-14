import fs from 'fs';
import net from 'net';
import { pipeline } from 'stream';
import shortid from 'https://cdn.skypack.dev/shortid';

const server = net.createServer((socket) => {
  const id = shortid();
  const writer = fs.createWriteStream(`${process.cwd()}/${id}.txt`);
  pipeline(socket, writer);
});

await server.listen(3000, '127.0.0.1');

console.log('Server listening on port 3000...');
