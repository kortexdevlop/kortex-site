import {spawnSync} from 'node:child_process';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {randomUUID,randomBytes} from 'node:crypto';
const root=resolve(import.meta.dirname,'..'),dir=resolve(root,'target/search-benchmark');mkdirSync(dir,{recursive:true});
function cli(args){const r=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,...(args[0]==='whoami'?[]:['--profile','kortex'])],{cwd:root,encoding:'utf8',windowsHide:true,timeout:180000,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'}});if(r.status!==0)throw Error(r.stderr.slice(-1000));return r.stdout;}
const owner=JSON.parse(readFileSync(resolve(root,'target/cloud-staging-resources.json'))),identity=JSON.parse(cli(['whoami','--json']));if(identity.accounts[0].id!==owner.account)throw Error('Account mismatch');const token=JSON.parse(cli(['auth','token','--json'])).token;
async function cf(method,path,body){const r=await fetch('https://api.cloudflare.com/client/v4/accounts/'+owner.account+path,{method,headers:{Authorization:'Bearer '+token,'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:AbortSignal.timeout(30000)});const v=await r.json();if(!r.ok||!v.success)throw Error(JSON.stringify(v.errors));return v.result;}
const name='kortex-search-bench-'+randomUUID().slice(0,8),report={date:new Date().toISOString(),rows:10000,environment:'temporary real Cloudflare D1/Worker; synthetic benchmark only',worker:name,samples_ms:[],passed:false,cleanup:false};let database,workerCreated=false;
try{
 database=await cf('POST','/d1/database',{name});report.database_id=database.uuid;
 const config={name,main:resolve(root,'api/src/index.ts'),compatibility_date:'2026-10-26',send_metrics:false,workers_dev:true,preview_urls:false,vars:{KORTEX_ENV:'staging',PUBLIC_WEB_ORIGIN:'https://staging.kortex.site',CURSOR_SIGNING_KEY:randomBytes(32).toString('hex')},d1_databases:[{binding:'DB',database_name:name,database_id:database.uuid,migrations_dir:resolve(root,'database/migrations')}],r2_buckets:[{binding:'TOOLS_BUCKET',bucket_name:owner.bucket}]};const path=resolve(dir,'config.json');writeFileSync(path,JSON.stringify(config));
 cli(['d1','migrations','apply','DB','--remote','--config',path]);
 const quote=v=>"'"+JSON.stringify(v).replaceAll("'","''")+"'",m=JSON.parse(readFileSync(resolve(root,'tools/calculatrice/manifest.json'))),c=JSON.parse(readFileSync(resolve(root,'tools/calculatrice/contract.json'))),d=JSON.parse(readFileSync(resolve(root,'tests/fixtures/artifact-signed-test.json')));
 const sql=`INSERT INTO categories VALUES('calculation','Calculation');
WITH RECURSIVE seq(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM seq WHERE n<10000) INSERT INTO tools(id,name,name_norm,summary,category_id,created_at,updated_at) SELECT printf('bench-%05d',n),printf('Benchmark %05d',n),printf('benchmark %05d',n),${"'"+m.summary.replaceAll("'","''")+"'"},'calculation',1,1 FROM seq;
INSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,created_at) SELECT id,'1.0.0',printf('%064x',1),'1.0','1.0',json_set(${quote(m)},'$.id',id,'$.name',name),json_set(${quote(c)},'$.tool_id',id),1 FROM tools;
INSERT INTO signing_keys VALUES('benchmark-test','${'A'.repeat(43)}=',1,0,9999999999999,0);
INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at) SELECT tool_id,version,sha256,'tools/'||tool_id||'/'||version||'/'||sha256||'.tool',100,'benchmark-test',json_set(${quote(d)},'$.tool_id',tool_id,'$.sha256',sha256,'$.signing_key_id','benchmark-test'),1 FROM tool_versions;
UPDATE artifacts SET head_byte_length=100,head_etag='BENCHMARK_TEST',read_byte_length=100,read_sha256=sha256,signature_verified_at=2,r2_verified_at=2;
INSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at) SELECT printf('94000000-0000-4000-8000-%012d',CAST(substr(tool_id,7) AS INTEGER)),sha256,tool_id,version,2,2 FROM artifacts;
UPDATE publication_jobs SET status='validated',updated_at=3;UPDATE publication_jobs SET status='published',updated_at=4;UPDATE tool_versions SET status='published';UPDATE tools SET status='published',published_version='1.0.0',published_at=1,updated_at=4;`;
 const seed=resolve(dir,'seed.sql');writeFileSync(seed,sql);cli(['d1','execute','DB','--remote','--config',path,'--file',seed]);cli(['deploy','--config',path]);workerCreated=true;
 const sub=await cf('GET','/workers/subdomain'),url=`https://${name}.${sub.subdomain}.workers.dev/api/v1/tools?q=benchmark&limit=8`;
 for(let i=0;i<5;i++){const r=await fetch(url);if(r.status===200)break;await new Promise(done=>setTimeout(done,1000));}
 for(let i=0;i<25;i++){const start=performance.now(),r=await fetch(url,{signal:AbortSignal.timeout(5000)}),data=await r.json();if(r.status!==200||data.items?.length!==8||!data.next_cursor)throw Error('Invalid benchmark response');report.samples_ms.push(Number((performance.now()-start).toFixed(2)));}
 const sorted=[...report.samples_ms].sort((a,b)=>a-b);report.p95_ms=sorted[Math.ceil(sorted.length*.95)-1];report.passed=report.p95_ms<500;console.log(JSON.stringify({rows:report.rows,p95_ms:report.p95_ms,passed:report.passed}));if(!report.passed)process.exitCode=1;
}catch(error){report.error=String(error.message).slice(0,1500);console.error(report.error);process.exitCode=1;}
finally{try{if(workerCreated)await cf('DELETE','/workers/scripts/'+name);if(database)await cf('DELETE','/d1/database/'+database.uuid);report.cleanup=true;}catch(error){report.cleanup_error=String(error.message);process.exitCode=1;}writeFileSync(resolve(root,'docs/evidence/search-staging-benchmark.json'),JSON.stringify(report,null,2)+'\n');}
