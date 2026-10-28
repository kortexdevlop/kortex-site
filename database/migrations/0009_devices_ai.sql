CREATE TABLE device_authorizations (
 id TEXT PRIMARY KEY,
 challenge TEXT NOT NULL CHECK(length(challenge)=43),
 state TEXT NOT NULL CHECK(length(state)=43),
 redirect_uri TEXT NOT NULL CHECK(length(redirect_uri)<200),
 expires_at INTEGER NOT NULL,
 user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
 code_hash TEXT UNIQUE,
 consumed_at INTEGER
);
CREATE INDEX device_authorizations_expiry ON device_authorizations(expires_at);
CREATE TABLE device_sessions (
 id TEXT PRIMARY KEY,
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 token_hash TEXT NOT NULL UNIQUE CHECK(length(token_hash)=64),
 created_at INTEGER NOT NULL,
 expires_at INTEGER NOT NULL,
 revoked_at INTEGER,
 label TEXT NOT NULL DEFAULT 'Kortex Desktop Windows'
);
CREATE INDEX device_sessions_user ON device_sessions(user_id);
CREATE TABLE ai_quota (
 bucket TEXT PRIMARY KEY,
 requests INTEGER NOT NULL CHECK(requests>=0),
 reserved_tokens INTEGER NOT NULL CHECK(reserved_tokens>=0),
 expires_at INTEGER NOT NULL,
 last_reservation TEXT
);
ALTER TABLE oidc_flows ADD COLUMN return_path TEXT NOT NULL DEFAULT '/app';
