import net from 'net';

// 1. Create a TCP server as async iterator.

async function handleConnection(socket) {
  for await (const data of socket) {
    socket.write(data);
  }
}

async function runServer() {
  const server = net.createServer();
  await server.listen(3000, '127.0.0.1');
  for await (const conn of server) {
    handleConnection(conn);
  }
}

// 2. Create a TCP server with a callback.

async function runServerWithCallback() {
  const server = net.createServer(async (socket) => {
    for await (const data of socket) {
      socket.write(data);
    }
  });
  await server.listen(3001, '127.0.0.1');
}

runServer();
runServerWithCallback();
