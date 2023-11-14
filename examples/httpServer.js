import http from 'http';

const server = http.createServer(async (req, res) => {
  await res.writeHead(200, { 'Content-Type': 'text/html' });
  await res.write(req.url);
  await res.end();
});

await server.listen(3000);

console.log('Server listening on port 3000...');
