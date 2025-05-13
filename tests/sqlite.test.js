import test from 'test';
import assert from 'assert';
import { Database } from 'sqlite';

const initSchema = `
  CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL
  ) STRICT;
`;

test('[SQLite] Insert and retrieve a user.', () => {
  const database = new Database(':memory:');
  database.exec(initSchema);

  const insert = database.prepare('INSERT INTO users (id, name) VALUES (?, ?)');
  insert.run(1, 'Alice');

  const select = database.prepare('SELECT name FROM users WHERE id = ?');
  const row = select.get(1);

  assert.equal(row.name, 'Alice');
});

test('[SQLite] Retrieve all users.', () => {
  const database = new Database(':memory:');
  database.exec(initSchema);

  const insert = database.prepare('INSERT INTO users (id, name) VALUES (?, ?)');
  insert.run(1, 'Alice');
  insert.run(2, 'Bob');

  const selectAll = database.prepare('SELECT * FROM users');
  const rows = selectAll.all();

  assert.equal(rows.length, 2);
  assert.objectEqual(rows[0], { id: 1, name: 'Alice' });
  assert.objectEqual(rows[1], { id: 2, name: 'Bob' });
});

test('[SQLite] Handle constraint violation.', () => {
  const database = new Database(':memory:');
  database.exec(initSchema);

  const insert = database.prepare('INSERT INTO users (id, name) VALUES (?, ?)');
  insert.run(1, 'Alice');

  try {
    insert.run(1, 'Bob');
    assert.true(false);
  } catch (e) {
    assert.instanceOf(e, Error);
  }
});
