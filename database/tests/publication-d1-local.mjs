import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {mkdirSync,mkdtempSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'../..');mkdirSync(resolve(root,'target'),{recursive:true});
const dir=mkdtempSync(resolve(root,'target/d1-publication-')),config=resolve(dir,'wrangler.json');
writeFileSync(config,JSON.stringify({name:'kortex-publication-local-test',compatibility_date:'2026-10-27',send_metrics:false,d1_databases:[{binding:'KORTEX_DB',database_name:'kortex-publication-local-test',database_id:'00000000-0000-4000-8000-000000000091',migrations_dir:resolve(root,'database/migrations')}]},null,2));
const calls=[];
function run(args,expectedError){
 if(!args.includes('--local')||args.includes('--remote'))throw Error('LOCAL_ONLY');
 const r=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),'d1',...args,'--config',config,'--persist-to',resolve(dir,'state')],{cwd:root,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'},encoding:'utf8'});
 calls.push({args,exit_code:r.status,stdout:r.stdout,stderr:r.stderr});
 if(expectedError){assert.notEqual(r.status,0);assert.match(r.stderr+r.stdout,new RegExp(expectedError));return}
 assert.equal(r.status,0,r.stderr||r.stdout);return r.stdout;
}
const sql=(command,error)=>run(['execute','kortex-publication-local-test','--local','--command',command,'--json'],error);
run(['migrations','apply','kortex-publication-local-test','--local']);run(['migrations','apply','kortex-publication-local-test','--local']);
const initial=JSON.parse(run(['execute','kortex-publication-local-test','--local','--file',resolve(root,'database/tests/publication.test.sql'),'--json']));
assert.equal(initial.flatMap(x=>x.results??[]).find(r=>r.idempotent_job_count!==undefined)?.idempotent_job_count,1);
sql("UPDATE publication_jobs SET status='validated',updated_at=110",'UNVERIFIED_OR_REVOKED_ARTIFACT');
sql("UPDATE tool_versions SET status='published' WHERE version='1.0.0'",'PUBLICATION_NOT_VERIFIED');
sql("INSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at) VALUES('91000000-0000-4000-8000-000000000001',printf('%064x',999),'calculatrice','1.0.0',100,100)",'IDEMPOTENCY_CONFLICT');
sql("INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at) SELECT 'missing-tool',version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at FROM artifacts WHERE version='1.0.0'",'NOT_FOUND');
sql("INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at) SELECT tool_id,version,printf('%064x',999),r2_key,byte_length,signing_key_id,descriptor_json,created_at FROM artifacts WHERE version='1.0.0'",'HASH_CONFLICT');
sql("UPDATE artifacts SET head_byte_length=byte_length,head_etag='test-etag',read_byte_length=byte_length,read_sha256=sha256,signature_verified_at=110,r2_verified_at=110 WHERE version='1.0.0'; UPDATE publication_jobs SET status='validated',updated_at=120 WHERE version='1.0.0'; UPDATE publication_jobs SET status='published',updated_at=130 WHERE version='1.0.0'; UPDATE tool_versions SET status='published' WHERE version='1.0.0'; UPDATE tools SET status='published',published_version='1.0.0',published_at=130 WHERE id='calculatrice'");
sql("UPDATE publication_jobs SET status='uploaded',updated_at=140 WHERE version='1.0.0'",'INVALID_PUBLICATION_TRANSITION');
sql("INSERT INTO revocations(request_id,tool_id,version,reason,created_at) VALUES('91000000-0000-4000-8000-000000000004','calculatrice','1.0.0','TEST revocation',150)",'UNPUBLISH_CURRENT_FIRST');
sql("UPDATE tools SET status='archived',published_version=NULL WHERE id='calculatrice'; INSERT INTO revocations(request_id,tool_id,version,reason,created_at) VALUES('91000000-0000-4000-8000-000000000004','calculatrice','1.0.0','TEST revocation',150)");
sql("UPDATE tools SET status='published',published_version='1.0.0' WHERE id='calculatrice'",'PUBLICATION_NOT_VERIFIED|CURRENT_VERSION_NOT_PUBLISHED');
sql('DELETE FROM publication_job_events','AUDIT_APPEND_ONLY');
const final=JSON.parse(sql("SELECT count(*) AS migrations FROM d1_migrations; SELECT status FROM publication_jobs; SELECT count(*) AS events FROM publication_job_events; PRAGMA foreign_key_check;"));
assert.equal(final[0].results[0].migrations,2);assert.equal(final[1].results[0].status,'revoked');assert.equal(final[2].results[0].events,4);assert.equal(final[3].results.length,0);
const report={local_only:true,remote_writes:0,migrations:2,idempotent_job_count:1,refusals:calls.filter(c=>c.exit_code!==0).length,final_status:'revoked',audit_events:4,foreign_key_errors:0,calls};writeFileSync(resolve(root,'target/publication-d1-report.json'),JSON.stringify(report,null,2));console.log(JSON.stringify({...report,calls:undefined}));
