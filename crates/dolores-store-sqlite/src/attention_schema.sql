BEGIN IMMEDIATE;
CREATE TABLE IF NOT EXISTS attention_preferences(id INTEGER PRIMARY KEY CHECK(id=1),data TEXT NOT NULL,omitted INTEGER NOT NULL DEFAULT 0);
CREATE TABLE IF NOT EXISTS attention_sources(run TEXT PRIMARY KEY REFERENCES runs(id) ON DELETE CASCADE,sequence INTEGER NOT NULL);
CREATE TABLE IF NOT EXISTS attention_notices(id TEXT PRIMARY KEY,session TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,run TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,sequence INTEGER NOT NULL,kind TEXT NOT NULL,created_at INTEGER NOT NULL,seen INTEGER NOT NULL DEFAULT 0,delivery TEXT NOT NULL,UNIQUE(run,sequence));
CREATE INDEX IF NOT EXISTS attention_notice_delivery ON attention_notices(delivery,seen);
-- Existing events stay in run history; installing this feature does not notify
-- about years of earlier work or regenerate notices that were already pruned.
INSERT OR IGNORE INTO attention_sources(run,sequence) SELECT id,json_extract(data,'$.sequence') FROM runs;
PRAGMA user_version=48;
COMMIT;
