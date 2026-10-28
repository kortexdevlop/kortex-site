-- 0003 is already deployed for admin operations: never rename migration history.
CREATE TABLE users (
 id TEXT PRIMARY KEY CHECK(length(id)=36 AND substr(id,9,1)='-' AND substr(id,14,1)='-' AND substr(id,19,1)='-' AND substr(id,24,1)='-'),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 sync_enabled INTEGER NOT NULL DEFAULT 0 CHECK(sync_enabled IN (0,1))
);
CREATE TABLE external_identities (
 issuer TEXT NOT NULL CHECK(length(issuer) BETWEEN 1 AND 2048),
 subject TEXT NOT NULL CHECK(length(subject) BETWEEN 1 AND 255),
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 PRIMARY KEY(issuer,subject)
);
CREATE INDEX identities_user ON external_identities(user_id);
CREATE TABLE sessions (
 token_hash TEXT PRIMARY KEY CHECK(length(token_hash)=64 AND token_hash NOT GLOB '*[^0-9a-f]*'),
 user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 expires_at INTEGER NOT NULL CHECK(expires_at>created_at AND expires_at<=created_at+604800000),
 revoked_at INTEGER CHECK(revoked_at IS NULL OR revoked_at>=created_at)
);
CREATE INDEX sessions_user ON sessions(user_id);
CREATE INDEX sessions_expiry ON sessions(expires_at);
CREATE TABLE auth_events (
 id TEXT PRIMARY KEY,
 event TEXT NOT NULL CHECK(event IN ('login_ok','login_failed','logout')),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 expires_at INTEGER NOT NULL CHECK(expires_at>created_at AND expires_at<=created_at+604800000)
);
CREATE INDEX auth_events_expiry ON auth_events(expires_at);
