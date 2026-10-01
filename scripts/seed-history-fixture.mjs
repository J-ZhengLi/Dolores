import fs from 'node:fs';
import path from 'node:path';
import { DatabaseSync } from 'node:sqlite';

const directory = process.argv[2];
if (!directory || !path.isAbsolute(directory)) throw Error('Use an absolute, fresh fixture directory.');
fs.mkdirSync(directory, { recursive: true });
const filename = path.join(directory, 'dolores.db');
if (fs.existsSync(filename)) throw Error('Fixture database already exists; refusing to replace it.');
const db = new DatabaseSync(filename);
try {
  // Legacy schema deliberately exercises additive migration of full history.
  db.exec(`CREATE TABLE sessions(id TEXT PRIMARY KEY,title TEXT NOT NULL,updated_at INTEGER NOT NULL);
    CREATE TABLE messages(id INTEGER PRIMARY KEY,session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,role TEXT NOT NULL,content TEXT NOT NULL);
    CREATE TABLE preferences(id INTEGER PRIMARY KEY CHECK(id=1),base_url TEXT NOT NULL,model TEXT NOT NULL);
    PRAGMA user_version=1; BEGIN;`);
  const session = db.prepare('INSERT INTO sessions VALUES(?,?,?)');
  session.run('long', 'Long conversation fixture', 10000);
  for (let i = 0; i < 136; i++) session.run(`fixture-${String(i).padStart(3, '0')}`, `Fixture chat ${i}`, 42);
  const message = db.prepare('INSERT INTO messages(session_id,role,content) VALUES(?,?,?)');
  for (let i = 0; i < 123; i++) {
    message.run('long', 'user', `你好 ${i}`);
    message.run('long', 'assistant', 'Complete fixture answer ' + i + '\n\n```\n<script>literal text</script>');
  }
  db.exec('COMMIT');
} finally { db.close(); }
