import { Database } from 'sqlite';

const database = new Database(':memory:');

const initSchema = `
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
`;

database.exec(initSchema);

const createUser = database.prepare(`
  INSERT INTO users (id, username, password, created_at)
  VALUES (?, ?, ?, ?)
  RETURNING id, username, created_at
`);

const getUsers = database.prepare(`
  SELECT * FROM users;
`);

const bob = createUser.get(1, "bob", "123", Date.now());
const alice = createUser.get(2, "alice", "987", Date.now());

console.log(bob, alice);
console.log(getUsers.all());

