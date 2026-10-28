CREATE TABLE oidc_flows (
 state_hash TEXT PRIMARY KEY CHECK(length(state_hash)=64 AND state_hash NOT GLOB '*[^0-9a-f]*'),
 browser_hash TEXT NOT NULL CHECK(length(browser_hash)=64),
 nonce_hash TEXT NOT NULL CHECK(length(nonce_hash)=64),
 verifier_encrypted TEXT NOT NULL CHECK(length(verifier_encrypted) BETWEEN 1 AND 512),
 config_hash TEXT NOT NULL CHECK(length(config_hash)=64),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 expires_at INTEGER NOT NULL CHECK(expires_at>created_at AND expires_at<=created_at+600000)
);
CREATE INDEX oidc_flows_expiry ON oidc_flows(expires_at);
