const stdout = process.stdout;

function render() {
  const { rows, columns } = stdout;
  const lines = [`Rows: ${rows}`, `Columns: ${columns}`];
  const width = Math.max(...lines.map((line) => line.length));
  const startRow = Math.floor((rows - lines.length) / 2);
  const startColumn = Math.floor((columns - width) / 2);

  stdout.cursorTo(0, 0);
  stdout.clearScreenDown();

  lines.forEach((line, index) => {
    stdout.cursorTo(startColumn, startRow + index);
    stdout.write(line);
  });
}

render();

stdout.on('resize', render);

process.on('SIGINT', () => {
  stdout.cursorTo(0, 0);
  stdout.clearScreenDown();
  process.exit(0);
});

// Needed to keep the event loop alive.
setInterval(() => {}, 1000);
