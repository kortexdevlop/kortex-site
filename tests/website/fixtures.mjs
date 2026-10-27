// Local integration data only. Never imported by the website or deployed API.
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {randomUUID} from 'node:crypto';
export const root=resolve(process.cwd());
export const wrangler=resolve(root,'node_modules/wrangler/bin/wrangler.js');
const literal=value=>"'"+String(typeof value==='object'?JSON.stringify(value):value).replaceAll("'","''")+"'";
export function executeSQL(sql){const file=resolve(root,'target/website-integration/query-'+randomUUID()+'.sql');writeFileSync(file,sql);const result=spawnSync(process.execPath,[wrangler,'d1','execute','DB','--local','--config',resolve(root,'target/website-integration/api.json'),'--persist-to',readFileSync(resolve(root,'target/website-integration/state-path.txt'),'utf8'),'--file',file],{cwd:root,encoding:'utf8'});if(result.status!==0)throw Error(result.stdout+result.stderr);}
export function toolSQL(id,name,{published=true,category='calculation',time=5}={}){
 const m=JSON.parse(readFileSync(resolve(root,'tools/calculatrice/manifest.json'))),c=JSON.parse(readFileSync(resolve(root,'tools/calculatrice/contract.json'))),f=JSON.parse(readFileSync(resolve(root,'target/website-integration/fixture.json')));
 Object.assign(m,{id,name,category});c.tool_id=id;const d={...f.descriptor,tool_id:id};
 // Non-calculator records exercise D1 state and SSR only, not artifact execution/signatures.
 let sql=`INSERT OR IGNORE INTO categories VALUES(${literal(category)},${literal(category)});\nINSERT INTO tools(id,name,name_norm,summary,category_id,created_at,updated_at) VALUES(${literal(id)},${literal(name)},${literal(name.toLowerCase())},${literal(m.summary)},${literal(category)},1,1);\nINSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,created_at) VALUES(${literal(id)},'1.0.0',${literal(d.sha256)},'1.0','1.0',${literal(m)},${literal(c)},1);\n`;
 if(!published)return sql;
 sql+=`INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at) VALUES(${literal(id)},'1.0.0',${literal(d.sha256)},${literal('tools/'+id+'/1.0.0/'+d.sha256+'.tool')},100,${literal(d.signing_key_id)},${literal(d)},1);\nUPDATE artifacts SET head_byte_length=100,head_etag='TEST_ATTESTATION',read_byte_length=100,read_sha256=sha256,signature_verified_at=2,r2_verified_at=2 WHERE tool_id=${literal(id)};\nINSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at) VALUES(${literal(randomUUID())},${literal(d.sha256)},${literal(id)},'1.0.0',2,2);\nUPDATE publication_jobs SET status='validated',updated_at=3 WHERE tool_id=${literal(id)};\nUPDATE publication_jobs SET status='published',updated_at=4 WHERE tool_id=${literal(id)};\nUPDATE tool_versions SET status='published' WHERE tool_id=${literal(id)};\nUPDATE tools SET status='published',published_version='1.0.0',published_at=${time},updated_at=4 WHERE id=${literal(id)};\n`;return sql;
}
