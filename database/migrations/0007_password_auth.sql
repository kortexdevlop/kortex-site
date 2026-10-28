CREATE TABLE password_credentials (
 email_key TEXT PRIMARY KEY CHECK(length(email_key)=64),
 email TEXT NOT NULL UNIQUE CHECK(length(email) BETWEEN 3 AND 254),
 user_id TEXT NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
 password_hash TEXT NOT NULL CHECK(length(password_hash) BETWEEN 50 AND 256),
 email_verified INTEGER NOT NULL DEFAULT 0 CHECK(email_verified IN (0,1)),
 created_at INTEGER NOT NULL CHECK(created_at>=0)
);
CREATE TABLE password_auth_forms (
 token_hash TEXT PRIMARY KEY CHECK(length(token_hash)=64),
 expires_at INTEGER NOT NULL CHECK(expires_at>=0)
);
CREATE INDEX password_forms_expiry ON password_auth_forms(expires_at);
CREATE TABLE password_auth_limits (
 bucket TEXT PRIMARY KEY CHECK(length(bucket)=64),
 attempts INTEGER NOT NULL CHECK(attempts>0),
 expires_at INTEGER NOT NULL CHECK(expires_at>=0)
);
CREATE INDEX password_limits_expiry ON password_auth_limits(expires_at);
