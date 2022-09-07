import net from 'net';

const HTTP_REQUEST =
  'GET / HTTP/1.1\r\nHost: rssweather.com\r\nConnection: close\r\n\r\n';

const client = net.createConnection({
  host: '104.21.45.178',
  port: 80,
});

client.setEncoding('utf-8');

client.on('connect', () => client.write(HTTP_REQUEST));

client.on('data', (data) => console.log(data));

client.on('close', () => console.log('Connection closed.'));
