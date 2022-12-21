import net from 'net';

const HTTP_REQUEST =
  'GET / HTTP/1.1\r\nHost: rssweather.com\r\nConnection: close\r\n\r\n';

// 1. Using events to handle data.

const client = await net.createConnection({
  host: '104.21.45.178',
  port: 80,
});

client.setEncoding('utf-8');

client.on('data', (data) => console.log(data));

client.on('close', () => console.log('Connection closed.'));

await client.write(HTTP_REQUEST);

// 2. Using async iterators to handle data

const client2 = await net.createConnection({
  host: '104.21.45.178',
  port: 80,
});

client2.setEncoding('utf-8');

await client2.write(HTTP_REQUEST);

for await (const data of client2) {
  console.log(data);
}

console.log('Connection closed.');
