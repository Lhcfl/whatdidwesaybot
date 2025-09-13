CREATE TABLE IF NOT EXISTS messages (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL,
  group_id INTEGER NOT NULL,
  message_id INTEGER NOT NULL,
  text TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
  text,
  content = 'messages',
  content_rowid = 'id'
);

CREATE TABLE IF NOT EXISTS global_messages (
  id INTEGER PRIMARY KEY,
  user_id INTEGER NOT NULL,
  text TEXT NOT NULL
);

CREATE VIRTUAL TABLE IF NOT EXISTS global_messages_fts USING fts5(
  text,
  content = 'global_messages',
  content_rowid = 'id'
);

CREATE TABLE IF NOT EXISTS configs (
  group_id INTEGER NOT NULL,
  user_id INTEGER NOT NULL,
  allow INTEGER NOT NULL,
  PRIMARY KEY (group_id, user_id),
  UNIQUE (group_id, user_id)
);

CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY,
  first_name TEXT NOT NULL,
  last_name TEXT,
  username TEXT
);