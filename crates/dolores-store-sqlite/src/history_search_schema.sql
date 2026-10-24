BEGIN IMMEDIATE;
CREATE TABLE IF NOT EXISTS history_organization (
    session TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
    revision INTEGER NOT NULL CHECK(revision>=1),
    pinned INTEGER NOT NULL CHECK(pinned IN (0,1)),
    archived INTEGER NOT NULL CHECK(archived IN (0,1))
);
CREATE VIRTUAL TABLE IF NOT EXISTS history_titles USING fts5(title,content='sessions',content_rowid='rowid',tokenize='trigram');
CREATE VIRTUAL TABLE IF NOT EXISTS history_messages USING fts5(content,content='messages',content_rowid='id',tokenize='trigram');
CREATE TRIGGER IF NOT EXISTS history_title_insert AFTER INSERT ON sessions BEGIN
    INSERT INTO history_titles(rowid,title) VALUES(new.rowid,new.title);
END;
CREATE TRIGGER IF NOT EXISTS history_title_delete AFTER DELETE ON sessions BEGIN
    INSERT INTO history_titles(history_titles,rowid,title) VALUES('delete',old.rowid,old.title);
END;
CREATE TRIGGER IF NOT EXISTS history_title_update AFTER UPDATE OF title ON sessions BEGIN
    INSERT INTO history_titles(history_titles,rowid,title) VALUES('delete',old.rowid,old.title);
    INSERT INTO history_titles(rowid,title) VALUES(new.rowid,new.title);
END;
CREATE TRIGGER IF NOT EXISTS history_message_insert AFTER INSERT ON messages BEGIN
    INSERT INTO history_messages(rowid,content) VALUES(new.id,new.content);
END;
CREATE TRIGGER IF NOT EXISTS history_message_delete AFTER DELETE ON messages BEGIN
    INSERT INTO history_messages(history_messages,rowid,content) VALUES('delete',old.id,old.content);
END;
CREATE TRIGGER IF NOT EXISTS history_message_update AFTER UPDATE OF content ON messages BEGIN
    INSERT INTO history_messages(history_messages,rowid,content) VALUES('delete',old.id,old.content);
    INSERT INTO history_messages(rowid,content) VALUES(new.id,new.content);
END;
INSERT INTO history_titles(history_titles) VALUES('rebuild');
INSERT INTO history_messages(history_messages) VALUES('rebuild');
PRAGMA user_version=47;
COMMIT;
