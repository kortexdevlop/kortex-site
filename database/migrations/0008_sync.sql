-- 0004..0007 are already deployed; migration history is immutable.
CREATE TABLE user_sync (
 user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
 library_revision INTEGER NOT NULL DEFAULT 0 CHECK(library_revision BETWEEN 0 AND 9007199254740990),
 preferences_revision INTEGER NOT NULL DEFAULT 0 CHECK(preferences_revision BETWEEN 0 AND 9007199254740990),
 library_mutation TEXT,
 preferences_mutation TEXT
);
CREATE TABLE user_library (
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 tool_id TEXT NOT NULL CHECK(length(tool_id) BETWEEN 3 AND 64),
 tag TEXT NOT NULL CHECK(length(tag)=36),
 added_revision INTEGER NOT NULL CHECK(added_revision>=0),
 removed_revision INTEGER CHECK(removed_revision IS NULL OR removed_revision>0),
 PRIMARY KEY(user_id,tag)
);
CREATE INDEX user_library_tools ON user_library(user_id,tool_id);
CREATE TABLE user_preferences (
 user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
 data_json TEXT NOT NULL CHECK(json_valid(data_json) AND length(data_json)<=2048)
);
CREATE TABLE sync_events (
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 entity TEXT NOT NULL CHECK(entity IN ('library','preferences')),
 revision INTEGER NOT NULL CHECK(revision>0),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 PRIMARY KEY(user_id,entity,revision)
);
