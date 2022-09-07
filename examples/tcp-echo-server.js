import net from 'net';

const server = net.createServer();

server.on('connection', async (socket) => {
  for await (const data of socket) {
    socket.write(data);
  }
});

server.listen(3000, '127.0.0.1', () => {
  console.log('Server is listening on port 3000...');
});
