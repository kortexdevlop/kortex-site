// Bounded authenticated HTTP probe of the actual telemetry module. No D1, R2,
// AI or production bindings, and no external notification is sent.
import {mkdirSync,writeFileSync,readFileSync} from 'node:fs';
import {resolve} from 'node:path';import {randomBytes} from 'node:crypto';import {spawnSync} from 'node:child_process';
import {cloudApi} from './release-cloud.mjs';import {HealthWindow} from '../api/src/lib/telemetry.ts';
const token=process.env.CLOUDFLARE_API_TOKEN,account=JSON.parse(readFileSync('infra/deploy/channels.json')).staging.account_id;if(!token)throw Error('CLOUDFLARE_CREDENTIAL_REQUIRED');
const name='kortex-observe-qa-'+randomBytes(4).toString('hex'),key=randomBytes(32).toString('hex'),dir=resolve('target/qa-telemetry',name);mkdirSync(dir,{recursive:true});
const cloud=cloudApi(token,account),expiry=Date.now()+1800000,report={date:new Date().toISOString(),owner:name,passed:false,cleanup:false,d1_bindings:0,r2_bindings:0,ai_bindings:0,notification_sent:false,http_requests:0};let created=false;
const entry=resolve(dir,'worker.ts');writeFileSync(entry,`import {telemetry} from '../../../api/src/lib/telemetry';
export default {async fetch(request:Request,env:any){if(!env.OPERATOR_KEY||request.headers.get('X-QA-Key')!==env.OPERATOR_KEY||Date.now()>Number(env.EXPIRES))return new Response(null,{status:404});const u=new URL(request.url);if(u.pathname!=='/qa/probe'||request.method!=='GET')return new Response(null,{status:404});const rows:any[]=[];const metric=telemetry(request,{KORTEX_ENV:'staging',TELEMETRY_ENABLED:'true'},r=>rows.push(r));const r=metric.finish(new Response(null,{status:u.searchParams.get('fault')==='1'?503:200}));return new Response(JSON.stringify(rows[0]),{status:r.status,headers:{'Content-Type':'application/json','Cache-Control':'no-store'}})}};`);
const config=resolve(dir,'wrangler.json');writeFileSync(config,JSON.stringify({name,main:entry,account_id:account,compatibility_date:'2026-10-26',workers_dev:true,preview_urls:false,send_metrics:false,observability:{enabled:false},vars:{QA_OWNER:name,EXPIRES:String(expiry)}}));
try{
 const deploy=spawnSync(process.execPath,['node_modules/wrangler/bin/wrangler.js','deploy','--config',config],{encoding:'utf8',windowsHide:true,timeout:120000,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'}});writeFileSync(resolve(dir,'deploy.log'),(deploy.stdout||'')+(deploy.stderr||''));if(deploy.status!==0)throw Error('QA_PROBE_DEPLOY_FAILED');created=true;
 await cloud('/workers/scripts/'+name+'/secrets','PUT',{name:'OPERATOR_KEY',text:key,type:'secret_text'});
 const sub=await cloud('/workers/subdomain'),origin='https://'+name+'.'+sub.subdomain+'.workers.dev';
 let ready=false;for(let attempt=0;attempt<30;attempt++){const r=await fetch(origin+'/qa/probe',{headers:{'X-QA-Key':key},signal:AbortSignal.timeout(10000)});report.http_requests++;if(r.status===200&&r.headers.get('content-type')?.includes('application/json')){ready=true;break}await new Promise(done=>setTimeout(done,1000))}if(!ready)throw Error('QA_DEPLOYMENT_NOT_READY');
 const denied=await fetch(origin+'/qa/probe',{signal:AbortSignal.timeout(10000)});report.http_requests++;if(denied.status!==404)throw Error('QA_ACCESS_GATE');
 const all=[];for(const faults of [3,0]){const health=new HealthWindow(),rows=[];for(let start=0;start<100;start+=5){const batch=await Promise.all(Array.from({length:5},async(_,i)=>{const n=start+i,r=await fetch(origin+'/qa/probe?fault='+(n<faults?'1':'0')+'&private=PRIVATE_FILE_QA',{headers:{'X-QA-Key':key,'X-Request-ID':'99999999-0000-4000-8000-000000000001'},signal:AbortSignal.timeout(10000)});report.http_requests++;const row=await r.json();if(row.status!==r.status||row.request_id==='99999999-0000-4000-8000-000000000001'||JSON.stringify(row).includes('PRIVATE_FILE'))throw Error('TELEMETRY_DISCLOSURE');return row}));rows.push(...batch);await new Promise(r=>setTimeout(r,200))}for(const row of rows)health.add(row.status,row.duration_ms,row.minute*60000);const summary=health.snapshot();if(summary.alert!==(faults===3)||summary.requests!==100)throw Error('ALERT_THRESHOLD');all.push({injected_503:faults,summary});}
 report.rounds=all;report.passed=true;
}finally{
 if(created){const settings=await cloud('/workers/scripts/'+name+'/settings');if(!settings.bindings.some(b=>b.name==='QA_OWNER'&&b.text===name))throw Error('QA_OWNER_MISMATCH');await cloud('/workers/scripts/'+name,'DELETE');report.cleanup=true}
 mkdirSync('docs/evidence',{recursive:true});writeFileSync('docs/evidence/telemetry-cloud-probe.json',JSON.stringify(report,null,2)+'\n');
}
console.log(JSON.stringify({passed:report.passed,cleanup:report.cleanup,http_requests:report.http_requests}));
