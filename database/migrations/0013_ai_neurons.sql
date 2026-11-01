CREATE TABLE ai_neurons (
 bucket TEXT PRIMARY KEY,
 used INTEGER NOT NULL CHECK(used>=0),
 expires_at INTEGER NOT NULL,
 reservation TEXT
);
CREATE INDEX ai_neurons_expiry ON ai_neurons(expires_at);
