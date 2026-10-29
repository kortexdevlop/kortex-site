import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {channels,context,protection} from './ci-publish-tool.mjs';
import {requireQuality} from './ci-quality.mjs';
const target=process.argv[2];if(!['api','runtime'].includes(target))throw Error('Explicit api or runtime target required');
if(process.env.CI==='true'){context(process.env,'staging');await requireQuality(process.env);await protection(process.env,'staging')}else if(process.platform!=='win32'||!process.argv.includes('--operator'))throw Error('Explicit Windows staging operator required');
const token=process.env.CLOUDFLARE_API_TOKEN;if(!token)throw Error('Cloudflare credential required');const c=channels.staging,worker=target==='api'?c.api_worker:c.runtime_worker,folder='target/ci-reports/rollback';mkdirSync(folder,{recursive:true});
async function cloud(path,method='GET',body){const r=await fetch(`https://api.cloudflare.com/client/v4/accounts/${c.account_id}${path}`,{method,headers:{Authorization:'Bearer '+token,'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),redirect:'error',signal:AbortSignal.timeout(20000)});const v=await r.json();if(!r.ok||!v.success)throw Error('Rollback Cloudflare request failed '+r.status);return v.result}
function cli(args){const r=spawnSync(process.execPath,[resolve('node_modules/wrangler/bin/wrangler.js'),...args],{encoding:'utf8',windowsHide:true,timeout:45000,maxBuffer:8*1024*1024,env:{...process.env,CLOUDFLARE_ACCOUNT_ID:c.account_id,WRANGLER_SEND_METRICS:'false'}});writeFileSync(folder+'/'+target+'-'+args[0]+'-'+Date.now()+'.log',((r.stdout||'')+(r.stderr||'')).replaceAll(token,'[REDACTED]'));if(r.status!==0)throw Error('Rollback command failed: '+args[0]);return r.stdout}
const history=(await cloud('/workers/scripts/'+worker+'/deployments')).deployments.filter(d=>d.versions?.length===1),current=history[0]?.versions[0].version_id,previous=history.find(d=>d.versions[0].version_id!==current)?.versions[0].version_id;if(![current,previous].every(v=>/^[a-f0-9-]{36}$/.test(v||'')))throw Error('Two concrete Worker versions required');
const settings=await cloud('/workers/scripts/'+worker+'/settings'),publicVars=b=>Object.fromEntries(b.filter(x=>x.type==='plain_text').map(x=>[x.name,x.text]));
const schema=async()=>{const result=await cloud('/d1/database/'+c.database_id+'/query','POST',{sql:"SELECT name,sql FROM sqlite_schema WHERE type IN ('table','index','trigger') ORDER BY name"});return createHash('sha256').update(JSON.stringify(result[0].results)).digest('hex')};const before=await schema();
if(target==='api'){const config=existsSync('target/ci-reports/deploy/api.json')?'target/ci-reports/deploy/api.json':'target/distribution-staging/api.json';cli(['d1','time-travel','info','DB','--json','--config',config])}
let rolledBack=false,restored=false;
try{cli(['rollback',previous,'--name',worker,'--yes','--message','Staging rollback rehearsal; no database restore']);rolledBack=true;const health=await fetch(c.api_origin+'/api/v1/health',{cache:'no-store',signal:AbortSignal.timeout(15000)});if(!health.ok)throw Error('Previous deployment health failed')}
finally{
 cli(['rollback',current,'--name',worker,'--yes','--message','Restore current staging deployment after rehearsal']);
 const form=new FormData();form.set('settings',JSON.stringify({bindings:settings.bindings}));const r=await fetch(`https://api.cloudflare.com/client/v4/accounts/${c.account_id}/workers/scripts/${worker}/settings`,{method:'PATCH',headers:{Authorization:'Bearer '+token},body:form,redirect:'error',signal:AbortSignal.timeout(20000)});if(!r.ok||!(await r.json()).success)throw Error('Settings restore failed');restored=true;
}
const afterSettings=await cloud('/workers/scripts/'+worker+'/settings'),after=await schema();if(JSON.stringify(publicVars(settings.bindings))!==JSON.stringify(publicVars(afterSettings.bindings))||before!==after)throw Error('Rollback changed settings or database schema');
const probe=await fetch(target==='api'?c.api_origin+'/api/v1/runtime/channels':c.runtime_origin+'/version.json',{cache:'no-store',signal:AbortSignal.timeout(15000)});if(!probe.ok)throw Error('Current runtime deployment not restored');
const proof={date:new Date().toISOString(),staging_only:true,target,worker,previous_version:previous,current_version:current,rollback_exercised:rolledBack,current_restored:restored,public_variables_preserved:true,database_schema_unchanged:true,database_restore:false,passed:true};writeFileSync(folder+'/'+target+'.json',JSON.stringify(proof,null,2)+'\n');console.log(JSON.stringify(proof));
