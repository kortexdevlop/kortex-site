-- An idempotency record and its mutation commit in the same D1 batch transaction.
CREATE TABLE admin_operations (
 request_id TEXT PRIMARY KEY NOT NULL,
 request_sha256 TEXT NOT NULL CHECK(length(request_sha256)=64),
 operation TEXT NOT NULL CHECK(operation IN ('publish','rollback','revoke')),
 tool_id TEXT NOT NULL,
 version TEXT NOT NULL,
 expected_current_version TEXT,
 subject TEXT NOT NULL,
 token_jti TEXT NOT NULL,
 response_json TEXT NOT NULL CHECK(json_valid(response_json) AND length(response_json)<=4096),
 created_at INTEGER NOT NULL,
 FOREIGN KEY(tool_id,version) REFERENCES tool_versions(tool_id,version) DEFERRABLE INITIALLY DEFERRED
);
CREATE TRIGGER admin_operation_no_replace BEFORE INSERT ON admin_operations
WHEN EXISTS(SELECT 1 FROM admin_operations WHERE request_id=NEW.request_id)
BEGIN SELECT RAISE(ABORT,'ADMIN_OPERATION_REPLAY'); END;
CREATE TRIGGER admin_operation_no_update BEFORE UPDATE ON admin_operations
BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER admin_operation_no_delete BEFORE DELETE ON admin_operations
BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER admin_rollback_guard BEFORE INSERT ON admin_operations WHEN NEW.operation='rollback' BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM tools WHERE id=NEW.tool_id AND status='published' AND published_version=NEW.expected_current_version)
 THEN RAISE(ABORT,'EXPECTED_VERSION_CONFLICT') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM tool_versions v JOIN artifacts a ON a.tool_id=v.tool_id AND a.version=v.version JOIN signing_keys k ON k.id=a.signing_key_id
 WHERE v.tool_id=NEW.tool_id AND v.version=NEW.version AND v.status='published' AND a.signature_verified_at IS NOT NULL AND a.r2_verified_at IS NOT NULL
 AND k.revoked=0 AND NEW.created_at>=k.not_before AND NEW.created_at<k.expires
 AND EXISTS(SELECT 1 FROM publication_jobs j WHERE j.tool_id=v.tool_id AND j.version=v.version AND j.status='published')
 AND NOT EXISTS(SELECT 1 FROM revocations r WHERE (r.tool_id=v.tool_id AND r.version=v.version) OR r.signing_key_id=k.id))
 THEN RAISE(ABORT,'ROLLBACK_UNAVAILABLE') END;
END;
