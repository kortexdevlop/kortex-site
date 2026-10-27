-- Synthetic state attestations; no R2 request or cryptographic verification is claimed here.
INSERT INTO categories VALUES('calculation','Calculation');
INSERT INTO tools(id,name,name_norm,summary,category_id,created_at,updated_at) VALUES('calculatrice','Calculatrice','calculatrice','State test','calculation',100,100);
INSERT INTO signing_keys VALUES('state-test-key','AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=',1,0,10000,0);
INSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,created_at)
VALUES('calculatrice','1.0.0',printf('%064x',1),'1.0','1.0','{"id":"calculatrice","version":"1.0.0"}','{"tool_id":"calculatrice"}',100),
('calculatrice','1.1.0',printf('%064x',2),'1.0','1.0','{"id":"calculatrice","version":"1.1.0"}','{"tool_id":"calculatrice"}',100),
('calculatrice','1.2.0',printf('%064x',3),'1.0','1.0','{"id":"calculatrice","version":"1.2.0"}','{"tool_id":"calculatrice"}',100);
INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at)
SELECT tool_id,version,sha256,'tools/'||tool_id||'/'||version||'/'||sha256||'.tool',123,'state-test-key',
json_object('schema_version',1,'tool_id',tool_id,'version',version,'abi_version','1.0','sha256',sha256,'signing_key_id','state-test-key','signature','SYNTHETIC_SQL_STATE_ONLY'),100 FROM tool_versions;
INSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at)
VALUES('91000000-0000-4000-8000-000000000001',printf('%064x',11),'calculatrice','1.0.0',100,100);
-- Exact request replay does not reset state or create another audit entry.
INSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at)
VALUES('91000000-0000-4000-8000-000000000001',printf('%064x',11),'calculatrice','1.0.0',100,100);
SELECT count(*) AS idempotent_job_count FROM publication_jobs;
SELECT count(*) AS idempotent_audit_count FROM publication_job_events;
PRAGMA foreign_key_check;
