ALTER TABLE user_sync ADD COLUMN workflows_revision INTEGER NOT NULL DEFAULT 0 CHECK(workflows_revision>=0);
ALTER TABLE user_sync ADD COLUMN workflows_mutation TEXT;
CREATE TABLE user_workflows(
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 workflow_id TEXT NOT NULL,
 definition_json TEXT,
 revision INTEGER NOT NULL CHECK(revision>0),
 updated_at INTEGER NOT NULL,
 PRIMARY KEY(user_id,workflow_id),
 CHECK(definition_json IS NULL OR json_valid(definition_json))
);
