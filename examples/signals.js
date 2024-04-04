import http from 'http';

// We need somehow to keep the event-loop alive.
http.createServer(() => {}).listen(3000);

let shouldExit = false;

// Exit on fast double CTRL+C key press.
const onSignal = () => {
  if (shouldExit) process.exit(0);
  shouldExit = true;
  setTimeout(() => {
    shouldExit = false;
  }, 500);
};

process.on('SIGINT', onSignal);
