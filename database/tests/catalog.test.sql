-- Synthetic data only. Run after 0001_catalog.sql, on an empty local test database.
INSERT INTO categories(id,name) VALUES('calculation','Calculation');
WITH RECURSIVE seq(n) AS (SELECT 1 UNION ALL SELECT n+1 FROM seq WHERE n<10000)
INSERT INTO tools(id,name,name_norm,summary,category_id,created_at,updated_at)
SELECT printf('sample-%05d',n),printf('Tool %05d',n),printf('tool %05d',n),'Synthetic catalog test','calculation',n,n FROM seq;
INSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,status,created_at)
SELECT id,'1.0.0',printf('%064x',created_at),'1.0','1.0',json_object('id',id,'version','1.0.0'),json_object('tool_id',id),'draft',created_at FROM tools;
INSERT INTO actions(tool_id,version,action_id,description,input_schema_json,output_schema_json,permissions_json,side_effect,idempotent,timeout_ms)
SELECT id,'1.0.0','multiply','Synthetic action','{"type":"object"}','{"type":"object"}','[]','none',1,1000 FROM tools;
UPDATE tool_versions SET status='published';
UPDATE tools SET status='published',published_version='1.0.0',published_at=created_at;
INSERT INTO publication_events(request_id,tool_id,version,event_type,created_at)
SELECT printf('00000000-0000-4000-8000-%012x',created_at),id,'1.0.0','synthetic-published',created_at FROM tools;
SELECT count(*) AS tool_count FROM tools WHERE status='published';
SELECT count(*) AS action_count FROM actions;
PRAGMA foreign_key_check;
EXPLAIN QUERY PLAN SELECT id FROM tools WHERE status='published' AND (published_at,id)<(5000,'sample-05000') ORDER BY published_at DESC,id DESC LIMIT 25;
EXPLAIN QUERY PLAN SELECT id FROM tools WHERE status='published' AND (name_norm,id)>('tool 05000','sample-05000') ORDER BY name_norm,id LIMIT 25;
EXPLAIN QUERY PLAN SELECT id FROM tools WHERE status='published' AND category_id='calculation' AND (published_at,id)<(5000,'sample-05000') ORDER BY published_at DESC,id DESC LIMIT 25;
EXPLAIN QUERY PLAN SELECT id FROM tools WHERE status='published' AND category_id='calculation' AND (name_norm,id)>('tool 05000','sample-05000') ORDER BY name_norm,id LIMIT 25;
