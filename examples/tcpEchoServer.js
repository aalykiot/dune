import net from 'net';

const server = net.createServer(async (socket) => {
  for await (const data of socket) {
    socket.write(data);
  }
});

console.log('Server is listening on port 3000...');

await server.listen(3000, '127.0.0.1');
