-- State and attestations only: the trusted publisher must actually verify R2 and Ed25519.
CREATE TABLE signing_keys (
 id TEXT PRIMARY KEY NOT NULL CHECK(length(id) BETWEEN 1 AND 64 AND id NOT GLOB '*[^A-Za-z0-9_-]*'),
 public_key_base64 TEXT NOT NULL CHECK(length(public_key_base64)=44),
 trust_version INTEGER NOT NULL CHECK(trust_version>=1),
 not_before INTEGER NOT NULL CHECK(not_before>=0),
 expires INTEGER NOT NULL CHECK(expires>not_before),
 revoked INTEGER NOT NULL DEFAULT 0 CHECK(revoked IN (0,1))
);
CREATE TABLE artifacts (
 tool_id TEXT NOT NULL,
 version TEXT NOT NULL,
 sha256 TEXT NOT NULL CHECK(length(sha256)=64 AND sha256 NOT GLOB '*[^0-9a-f]*'),
 r2_key TEXT NOT NULL UNIQUE CHECK(r2_key='tools/'||tool_id||'/'||version||'/'||sha256||'.tool'),
 byte_length INTEGER NOT NULL CHECK(byte_length BETWEEN 1 AND 26214400),
 signing_key_id TEXT NOT NULL REFERENCES signing_keys(id) ON DELETE RESTRICT,
 descriptor_json TEXT NOT NULL CHECK(length(CAST(descriptor_json AS BLOB))<=1048576 AND json_valid(descriptor_json)
  AND json_type(descriptor_json,'$.schema_version') IS 'integer' AND json_extract(descriptor_json,'$.schema_version')=1
  AND json_type(descriptor_json,'$.tool_id') IS 'text' AND json_extract(descriptor_json,'$.tool_id')=tool_id
  AND json_type(descriptor_json,'$.version') IS 'text' AND json_extract(descriptor_json,'$.version')=version
  AND json_type(descriptor_json,'$.sha256') IS 'text' AND json_extract(descriptor_json,'$.sha256')=sha256
  AND json_type(descriptor_json,'$.signing_key_id') IS 'text' AND json_extract(descriptor_json,'$.signing_key_id')=signing_key_id),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 head_byte_length INTEGER,
 head_etag TEXT CHECK(head_etag IS NULL OR length(head_etag) BETWEEN 1 AND 256),
 read_byte_length INTEGER,
 read_sha256 TEXT,
 signature_verified_at INTEGER,
 r2_verified_at INTEGER,
 PRIMARY KEY(tool_id,version),
 FOREIGN KEY(tool_id,version) REFERENCES tool_versions(tool_id,version) ON DELETE RESTRICT,
 CHECK(signature_verified_at IS NULL OR signature_verified_at>=created_at),
 CHECK((signature_verified_at IS NULL)=(r2_verified_at IS NULL)),
 CHECK(r2_verified_at IS NULL OR (r2_verified_at>=created_at AND head_byte_length IS NOT NULL AND head_byte_length=byte_length
  AND head_etag IS NOT NULL AND read_byte_length IS NOT NULL AND read_byte_length=byte_length AND read_sha256 IS NOT NULL AND read_sha256=sha256))
);
CREATE TABLE publication_jobs (
 request_id TEXT PRIMARY KEY NOT NULL CHECK(length(request_id)=36 AND substr(request_id,9,1)='-' AND substr(request_id,14,1)='-' AND substr(request_id,19,1)='-' AND substr(request_id,24,1)='-' AND length(replace(request_id,'-',''))=32 AND replace(request_id,'-','') NOT GLOB '*[^0-9a-fA-F]*'),
 request_sha256 TEXT NOT NULL CHECK(length(request_sha256)=64 AND request_sha256 NOT GLOB '*[^0-9a-f]*'),
 tool_id TEXT NOT NULL,
 version TEXT NOT NULL,
 status TEXT NOT NULL DEFAULT 'uploaded' CHECK(status IN ('uploaded','validated','published','failed','revoked')),
 error_code TEXT CHECK(error_code IS NULL OR length(error_code) BETWEEN 1 AND 64),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 updated_at INTEGER NOT NULL CHECK(updated_at>=created_at),
 FOREIGN KEY(tool_id,version) REFERENCES artifacts(tool_id,version) ON DELETE RESTRICT,
 CHECK(status!='failed' OR error_code IS NOT NULL)
);
CREATE UNIQUE INDEX idx_publication_jobs_active_version ON publication_jobs(tool_id,version) WHERE status IN ('uploaded','validated','published');
CREATE INDEX idx_publication_jobs_status ON publication_jobs(status,updated_at,request_id);
CREATE TABLE publication_job_events (
 id INTEGER PRIMARY KEY,
 request_id TEXT NOT NULL REFERENCES publication_jobs(request_id) ON DELETE RESTRICT,
 previous_status TEXT,
 status TEXT NOT NULL CHECK(status IN ('uploaded','validated','published','failed','revoked')),
 created_at INTEGER NOT NULL,
 error_code TEXT,
 UNIQUE(request_id,status)
);
CREATE INDEX idx_artifacts_signing_key ON artifacts(signing_key_id,tool_id,version);
CREATE TABLE revocations (
 request_id TEXT PRIMARY KEY NOT NULL CHECK(length(request_id)=36 AND substr(request_id,9,1)='-' AND substr(request_id,14,1)='-' AND substr(request_id,19,1)='-' AND substr(request_id,24,1)='-' AND length(replace(request_id,'-',''))=32 AND replace(request_id,'-','') NOT GLOB '*[^0-9a-fA-F]*'),
 tool_id TEXT,
 version TEXT,
 signing_key_id TEXT REFERENCES signing_keys(id) ON DELETE RESTRICT,
 reason TEXT NOT NULL CHECK(length(reason) BETWEEN 1 AND 1000),
 created_at INTEGER NOT NULL CHECK(created_at>=0),
 CHECK((tool_id IS NOT NULL AND version IS NOT NULL AND signing_key_id IS NULL) OR (tool_id IS NULL AND version IS NULL AND signing_key_id IS NOT NULL)),
 FOREIGN KEY(tool_id,version) REFERENCES artifacts(tool_id,version) ON DELETE RESTRICT,
 UNIQUE(tool_id,version),
 UNIQUE(signing_key_id)
);
CREATE INDEX idx_revocations_created ON revocations(created_at,request_id);
CREATE TRIGGER artifact_insert_guard BEFORE INSERT ON artifacts BEGIN
 SELECT CASE WHEN NEW.r2_verified_at IS NOT NULL OR NEW.signature_verified_at IS NOT NULL THEN RAISE(ABORT,'UPLOAD_FIRST') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM tools WHERE id=NEW.tool_id) THEN RAISE(ABORT,'NOT_FOUND') END;
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM tool_versions WHERE tool_id=NEW.tool_id AND version=NEW.version) THEN RAISE(ABORT,'VERSION_NOT_FOUND') END;
 SELECT CASE WHEN EXISTS(SELECT 1 FROM tool_versions WHERE tool_id=NEW.tool_id AND version=NEW.version AND sha256!=NEW.sha256) THEN RAISE(ABORT,'HASH_CONFLICT') END;
END;
CREATE TRIGGER artifact_identity_immutable BEFORE UPDATE OF tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at ON artifacts
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_ARTIFACT'); END;
CREATE TRIGGER artifact_evidence_immutable BEFORE UPDATE ON artifacts
WHEN OLD.r2_verified_at IS NOT NULL OR OLD.signature_verified_at IS NOT NULL
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_VERIFICATION'); END;
CREATE TRIGGER artifact_no_delete BEFORE DELETE ON artifacts BEGIN SELECT RAISE(ABORT,'IMMUTABLE_ARTIFACT'); END;
CREATE TRIGGER publication_job_idempotent BEFORE INSERT ON publication_jobs
WHEN EXISTS(SELECT 1 FROM publication_jobs WHERE request_id=NEW.request_id) BEGIN
 SELECT CASE WHEN EXISTS(SELECT 1 FROM publication_jobs WHERE request_id=NEW.request_id AND (request_sha256!=NEW.request_sha256 OR tool_id!=NEW.tool_id OR version!=NEW.version)) THEN RAISE(ABORT,'IDEMPOTENCY_CONFLICT') END;
 SELECT RAISE(IGNORE);
END;
CREATE TRIGGER publication_job_initial BEFORE INSERT ON publication_jobs WHEN NEW.status!='uploaded'
BEGIN SELECT RAISE(ABORT,'UPLOAD_FIRST'); END;
CREATE TRIGGER publication_job_identity BEFORE UPDATE OF request_id,request_sha256,tool_id,version,created_at ON publication_jobs
BEGIN SELECT RAISE(ABORT,'IMMUTABLE_PUBLICATION_REQUEST'); END;
CREATE TRIGGER publication_job_transition BEFORE UPDATE ON publication_jobs BEGIN
 SELECT CASE WHEN OLD.status IN ('failed','revoked') THEN RAISE(ABORT,'INVALID_PUBLICATION_TRANSITION') END;
 SELECT CASE WHEN NEW.updated_at<OLD.updated_at THEN RAISE(ABORT,'TIMESTAMP_ROLLBACK') END;
 SELECT CASE WHEN NEW.status!=OLD.status AND NOT (
  (OLD.status='uploaded' AND NEW.status IN ('validated','failed','revoked')) OR
  (OLD.status='validated' AND NEW.status IN ('published','failed','revoked')) OR
  (OLD.status='published' AND NEW.status='revoked')) THEN RAISE(ABORT,'INVALID_PUBLICATION_TRANSITION') END;
END;
CREATE TRIGGER publication_job_validation BEFORE UPDATE OF status ON publication_jobs
WHEN NEW.status IN ('validated','published') BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM artifacts a JOIN signing_keys k ON k.id=a.signing_key_id
  WHERE a.tool_id=NEW.tool_id AND a.version=NEW.version AND a.r2_verified_at IS NOT NULL AND a.signature_verified_at IS NOT NULL
   AND a.r2_verified_at<=NEW.updated_at AND a.signature_verified_at<=NEW.updated_at
   AND k.revoked=0 AND NEW.updated_at>=k.not_before AND NEW.updated_at<k.expires
   AND NOT EXISTS(SELECT 1 FROM revocations r WHERE (r.tool_id=a.tool_id AND r.version=a.version) OR r.signing_key_id=k.id))
  THEN RAISE(ABORT,'UNVERIFIED_OR_REVOKED_ARTIFACT') END;
END;
CREATE TRIGGER publication_job_no_delete BEFORE DELETE ON publication_jobs BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER publication_job_audit_insert AFTER INSERT ON publication_jobs BEGIN
 INSERT INTO publication_events(request_id,tool_id,version,event_type,created_at,detail_json)
 VALUES(NEW.request_id,NEW.tool_id,NEW.version,'publication-uploaded',NEW.created_at,json_object('request_sha256',NEW.request_sha256));
 INSERT INTO publication_job_events(request_id,status,created_at) VALUES(NEW.request_id,NEW.status,NEW.created_at);
END;
CREATE TRIGGER publication_job_audit_update AFTER UPDATE OF status ON publication_jobs WHEN NEW.status!=OLD.status BEGIN
 INSERT INTO publication_job_events(request_id,previous_status,status,created_at,error_code)
 VALUES(NEW.request_id,OLD.status,NEW.status,NEW.updated_at,NEW.error_code);
END;
CREATE TRIGGER publication_job_events_no_update BEFORE UPDATE ON publication_job_events BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER publication_job_events_no_delete BEFORE DELETE ON publication_job_events BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER published_version_requires_job BEFORE UPDATE OF status ON tool_versions WHEN NEW.status='published' BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM publication_jobs WHERE tool_id=NEW.tool_id AND version=NEW.version AND status='published') THEN RAISE(ABORT,'PUBLICATION_NOT_VERIFIED') END;
END;
CREATE TRIGGER published_pointer_requires_job BEFORE UPDATE OF status,published_version ON tools WHEN NEW.status='published' BEGIN
 SELECT CASE WHEN NOT EXISTS(SELECT 1 FROM publication_jobs j JOIN artifacts a USING(tool_id,version) JOIN signing_keys k ON k.id=a.signing_key_id
  WHERE j.tool_id=NEW.id AND j.version=NEW.published_version AND j.status='published' AND k.revoked=0
   AND NOT EXISTS(SELECT 1 FROM revocations r WHERE (r.tool_id=a.tool_id AND r.version=a.version) OR r.signing_key_id=k.id)) THEN RAISE(ABORT,'PUBLICATION_NOT_VERIFIED') END;
END;
CREATE TRIGGER signing_key_identity BEFORE UPDATE OF id,public_key_base64,trust_version,not_before,expires ON signing_keys BEGIN SELECT RAISE(ABORT,'IMMUTABLE_SIGNING_KEY'); END;
CREATE TRIGGER signing_key_revocation BEFORE UPDATE OF revoked ON signing_keys
WHEN NEW.revoked!=1 OR OLD.revoked=1 OR NOT EXISTS(SELECT 1 FROM revocations WHERE signing_key_id=OLD.id)
BEGIN SELECT RAISE(ABORT,'AUDITED_REVOCATION_REQUIRED'); END;
CREATE TRIGGER signing_key_no_delete BEFORE DELETE ON signing_keys BEGIN SELECT RAISE(ABORT,'IMMUTABLE_SIGNING_KEY'); END;
CREATE TRIGGER revocation_no_update BEFORE UPDATE ON revocations BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER revocation_no_delete BEFORE DELETE ON revocations BEGIN SELECT RAISE(ABORT,'AUDIT_APPEND_ONLY'); END;
CREATE TRIGGER revocation_current_guard BEFORE INSERT ON revocations
WHEN EXISTS(SELECT 1 FROM tools t JOIN artifacts a ON a.tool_id=t.id AND a.version=t.published_version
 WHERE t.status='published' AND ((a.tool_id=NEW.tool_id AND a.version=NEW.version) OR a.signing_key_id=NEW.signing_key_id))
BEGIN SELECT RAISE(ABORT,'UNPUBLISH_CURRENT_FIRST'); END;
CREATE TRIGGER revocation_apply AFTER INSERT ON revocations BEGIN
 UPDATE signing_keys SET revoked=1 WHERE id=NEW.signing_key_id;
 UPDATE publication_jobs SET status='revoked',updated_at=max(updated_at,NEW.created_at) WHERE status IN ('uploaded','validated','published') AND
  EXISTS(SELECT 1 FROM artifacts a WHERE a.tool_id=publication_jobs.tool_id AND a.version=publication_jobs.version AND ((a.tool_id=NEW.tool_id AND a.version=NEW.version) OR a.signing_key_id=NEW.signing_key_id));
 UPDATE tool_versions SET status='revoked' WHERE EXISTS(SELECT 1 FROM artifacts a WHERE a.tool_id=tool_versions.tool_id AND a.version=tool_versions.version AND ((a.tool_id=NEW.tool_id AND a.version=NEW.version) OR a.signing_key_id=NEW.signing_key_id));
END;
