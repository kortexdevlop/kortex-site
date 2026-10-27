-- Forward-only D1 migration; Wrangler records its application in d1_migrations.
CREATE TABLE categories (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) BETWEEN 3 AND 64 AND id NOT GLOB '*[^a-z0-9-]*'),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 100)
);
CREATE TABLE tools (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) BETWEEN 3 AND 64 AND id NOT GLOB '*[^a-z0-9-]*' AND substr(id,1,1)!='-' AND substr(id,-1)!='-' AND instr(id,'--')=0),
 name TEXT NOT NULL CHECK(length(name) BETWEEN 1 AND 100),
 name_norm TEXT NOT NULL CHECK(length(name_norm) BETWEEN 1 AND 200),
 summary TEXT NOT NULL CHECK(length(summary) BETWEEN 1 AND 240),
 category_id TEXT NOT NULL REFERENCES categories(id) ON DELETE RESTRICT,
 status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','published','archived')),
 published_version TEXT,
 published_at INTEGER CHECK(published_at IS NULL OR published_at>=0),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 updated_at INTEGER NOT NULL CHECK(updated_at>=created_at),
 CHECK(status!='published' OR (published_version IS NOT NULL AND published_at IS NOT NULL)),
 FOREIGN KEY(id,published_version) REFERENCES tool_versions(tool_id,version) DEFERRABLE INITIALLY DEFERRED
);
CREATE TABLE tool_versions (
 tool_id TEXT NOT NULL REFERENCES tools(id) ON DELETE RESTRICT,
 version TEXT NOT NULL CHECK(length(version) BETWEEN 5 AND 128),
 sha256 TEXT NOT NULL CHECK(length(sha256)=64 AND sha256 NOT GLOB '*[^0-9a-f]*'),
 abi_version TEXT NOT NULL CHECK(abi_version='1.0'),
 ui_schema_version TEXT NOT NULL CHECK(ui_schema_version='1.0'),
 manifest_json TEXT NOT NULL CHECK(length(CAST(manifest_json AS BLOB))<=1048576 AND json_valid(manifest_json) AND json_type(manifest_json,'$.id') IS 'text' AND json_extract(manifest_json,'$.id')=tool_id AND json_type(manifest_json,'$.version') IS 'text' AND json_extract(manifest_json,'$.version')=version),
 contract_json TEXT NOT NULL CHECK(length(CAST(contract_json AS BLOB))<=1048576 AND json_valid(contract_json) AND json_type(contract_json,'$.tool_id') IS 'text' AND json_extract(contract_json,'$.tool_id')=tool_id),
 status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft','published','revoked')),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 PRIMARY KEY(tool_id,version)
);
CREATE TABLE actions (
 tool_id TEXT NOT NULL,
 version TEXT NOT NULL,
 action_id TEXT NOT NULL CHECK(length(action_id) BETWEEN 1 AND 64 AND action_id NOT GLOB '*[^a-z0-9_-]*' AND substr(action_id,1,1) GLOB '[a-z]'),
 description TEXT NOT NULL CHECK(length(description) BETWEEN 1 AND 1000),
 input_schema_json TEXT NOT NULL CHECK(json_valid(input_schema_json) AND json_type(input_schema_json)='object'),
 output_schema_json TEXT NOT NULL CHECK(json_valid(output_schema_json) AND json_type(output_schema_json)='object'),
 permissions_json TEXT NOT NULL CHECK(json_valid(permissions_json) AND json_type(permissions_json)='array'),
 side_effect TEXT NOT NULL CHECK(side_effect IN ('none','local-write','network')),
 idempotent INTEGER NOT NULL CHECK(idempotent IN (0,1)),
 timeout_ms INTEGER NOT NULL CHECK(timeout_ms BETWEEN 1 AND 30000),
 PRIMARY KEY(tool_id,version,action_id),
 FOREIGN KEY(tool_id,version) REFERENCES tool_versions(tool_id,version) ON DELETE RESTRICT
);
CREATE TABLE publication_events (
 id INTEGER PRIMARY KEY,
 request_id TEXT NOT NULL UNIQUE CHECK(length(request_id)=36 AND length(replace(request_id,'-',''))=32 AND substr(request_id,9,1)='-' AND substr(request_id,14,1)='-' AND substr(request_id,19,1)='-' AND substr(request_id,24,1)='-' AND replace(request_id,'-','') NOT GLOB '*[^0-9a-fA-F]*'),
 tool_id TEXT NOT NULL,
 version TEXT NOT NULL,
 event_type TEXT NOT NULL CHECK(length(event_type) BETWEEN 1 AND 64),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 detail_json TEXT NOT NULL DEFAULT '{}' CHECK(length(CAST(detail_json AS BLOB))<=16384 AND json_valid(detail_json) AND json_type(detail_json)='object'),
 FOREIGN KEY(tool_id,version) REFERENCES tool_versions(tool_id,version) ON DELETE RESTRICT
);
CREATE INDEX idx_tools_recent ON tools(published_at DESC,id DESC) WHERE status='published';
CREATE INDEX idx_tools_name ON tools(name_norm ASC,id ASC) WHERE status='published';
CREATE INDEX idx_tools_category_recent ON tools(category_id,published_at DESC,id DESC) WHERE status='published';
CREATE INDEX idx_tools_category_name ON tools(category_id,name_norm ASC,id ASC) WHERE status='published';
CREATE INDEX idx_tools_category_fk ON tools(category_id);
CREATE INDEX idx_tool_versions_status ON tool_versions(status,tool_id,version);
CREATE INDEX idx_publication_events_tool ON publication_events(tool_id,created_at DESC,id DESC);
CREATE TRIGGER tools_id_immutable BEFORE UPDATE OF id ON tools WHEN NEW.id!=OLD.id
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_TOOL_ID'); END;
CREATE TRIGGER version_content_immutable BEFORE UPDATE OF tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json ON tool_versions
WHEN NEW.tool_id!=OLD.tool_id OR NEW.version!=OLD.version OR NEW.sha256!=OLD.sha256 OR NEW.abi_version!=OLD.abi_version OR NEW.ui_schema_version!=OLD.ui_schema_version OR NEW.manifest_json!=OLD.manifest_json OR NEW.contract_json!=OLD.contract_json
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_TOOL_VERSION'); END;
CREATE TRIGGER current_version_must_be_published BEFORE UPDATE OF status,published_version ON tools
WHEN NEW.status='published' AND NOT EXISTS(SELECT 1 FROM tool_versions WHERE tool_id=NEW.id AND version=NEW.published_version AND status='published')
BEGIN SELECT RAISE(ABORT,'CURRENT_VERSION_NOT_PUBLISHED'); END;
CREATE TRIGGER inserted_current_version_must_be_published BEFORE INSERT ON tools
WHEN NEW.status='published'
BEGIN SELECT RAISE(ABORT,'CREATE_DRAFT_FIRST'); END;
CREATE TRIGGER current_version_cannot_be_revoked BEFORE UPDATE OF status ON tool_versions
WHEN NEW.status!='published' AND EXISTS(SELECT 1 FROM tools WHERE id=OLD.tool_id AND status='published' AND published_version=OLD.version)
BEGIN SELECT RAISE(ABORT,'UNPUBLISH_CURRENT_FIRST'); END;
CREATE TRIGGER version_status_forward_only BEFORE UPDATE OF status ON tool_versions
WHEN (OLD.status='published' AND NEW.status NOT IN ('published','revoked')) OR (OLD.status='revoked' AND NEW.status!='revoked')
BEGIN SELECT RAISE(ABORT,'VERSION_STATUS_FORWARD_ONLY'); END;
CREATE TRIGGER publication_events_no_update BEFORE UPDATE ON publication_events
BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER publication_events_no_delete BEFORE DELETE ON publication_events
BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER tools_no_delete BEFORE DELETE ON tools
BEGIN SELECT RAISE(ABORT,'ARCHIVE_TOOL_INSTEAD'); END;
CREATE TRIGGER version_no_delete BEFORE DELETE ON tool_versions
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_TOOL_VERSION'); END;
CREATE TRIGGER published_actions_no_insert BEFORE INSERT ON actions
WHEN EXISTS(SELECT 1 FROM tool_versions WHERE tool_id=NEW.tool_id AND version=NEW.version AND status!='draft')
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_PUBLISHED_ACTIONS'); END;
CREATE TRIGGER published_actions_no_update BEFORE UPDATE ON actions
WHEN EXISTS(SELECT 1 FROM tool_versions WHERE tool_id=OLD.tool_id AND version=OLD.version AND status!='draft') OR EXISTS(SELECT 1 FROM tool_versions WHERE tool_id=NEW.tool_id AND version=NEW.version AND status!='draft')
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_PUBLISHED_ACTIONS'); END;
CREATE TRIGGER published_actions_no_delete BEFORE DELETE ON actions
WHEN EXISTS(SELECT 1 FROM tool_versions WHERE tool_id=OLD.tool_id AND version=OLD.version AND status!='draft')
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_PUBLISHED_ACTIONS'); END;
