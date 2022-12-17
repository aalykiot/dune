import net from 'net';

const HTTP_REQUEST =
  'GET / HTTP/1.1\r\nHost: rssweather.com\r\nConnection: close\r\n\r\n';

const client = await net.createConnection({
  host: '104.21.45.178',
  port: 80,
});

client.setEncoding('utf-8');
client.write(HTTP_REQUEST);

for await (const data of client) {
  console.log(data);
}

console.log('Connection closed.');
