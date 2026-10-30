// Provision isolated storage and render deployment configs; never deploy Workers,
// change DNS/mail, enable AI, buy a plan, or copy staging secrets/trust.
import {readFileSync,writeFileSync,mkdirSync,readdirSync} from 'node:fs';
import {resolve} from 'node:path';import {pathToFileURL} from 'node:url';import {createHash} from 'node:crypto';
import {cloudApi} from './release-cloud.mjs';
import {d1Budget} from './d1-test-budget.mjs';
export function productionPlan(channels,state){
 if(state.domain!=='kortex.site'||state.owner_decisions?.monthly_budget_eur!==0||state.owner_decisions?.automatic_paid_upgrade!==false)throw Error('PRODUCTION_ZERO_BUDGET_REQUIRED');
 const account=channels.staging?.account_id;if(!/^[a-f0-9]{32}$/.test(account??''))throw Error('PRODUCTION_ACCOUNT_REQUIRED');
 return {schema_version:1,account_id:account,database_name:'kortex-catalog-production',bucket:'kortex-tools-production',api_worker:'kortex-api-production',web_worker:'kortex-web-production',runtime_worker:'kortex-run-production',api_origin:'https://api.kortex.site',web_origin:'https://kortex.site',runtime_origin:'https://run.kortex.site',budget_eur:0,ai_enabled:false,google_enabled:false,dns_changes:false,mail_changes:false,worker_deployments:false};
}
export function authorizeStorageSetup(state,plan){if(state.billing_free_only_verified!==true&&!(state.owner_decisions?.workers_plan_confirmed==='free'&&state.storage_setup_zero_cost_verified===true))throw Error('FREE_PLAN_VERIFICATION_REQUIRED');if(plan.budget_eur!==0||plan.worker_deployments||plan.dns_changes||plan.mail_changes)throw Error('STORAGE_ONLY_SETUP_REQUIRED')}
export function renderProductionConfigs(p){
 if(!/^[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}$/.test(p.database_id??'')||p.database_id.startsWith('00000000-'))throw Error('REAL_PRODUCTION_DATABASE_REQUIRED');
 const common={compatibility_date:'2026-10-26',compatibility_flags:['nodejs_compat'],workers_dev:false,preview_urls:false,send_metrics:false,observability:{enabled:false}};
 return {api:{...common,name:p.api_worker,main:resolve('api/src/index.ts'),vars:{KORTEX_ENV:'production',PUBLIC_WEB_ORIGIN:p.web_origin,PASSWORD_AUTH_ENABLED:'true',OIDC_ENABLED:'false',AI_ENABLED:'false',TELEMETRY_ENABLED:'false'},d1_databases:[{binding:'DB',database_name:p.database_name,database_id:p.database_id,migrations_dir:resolve('database/migrations')}],r2_buckets:[{binding:'TOOLS_BUCKET',bucket_name:p.bucket}]},runtime:{...common,name:p.runtime_worker,main:resolve('runtime/web/worker.ts'),assets:{directory:resolve('target/web-dist'),binding:'ASSETS',run_worker_first:true},services:[{binding:'API_WORKER',service:p.api_worker}],vars:{PUBLIC_WEB_ORIGIN:p.web_origin,RUN_ORIGIN:p.runtime_origin}},website:{...common,name:p.web_worker,main:resolve('website/dist/server/entry.mjs'),no_bundle:true,assets:{directory:resolve('website/dist/client'),binding:'ASSETS'},services:[{binding:'API_WORKER',service:p.api_worker},{binding:'SITEMAP_WORKER',service:p.api_worker,entrypoint:'SitemapService'},{binding:'AUTH_WORKER',service:p.api_worker,entrypoint:'AuthService'}],vars:{WEB_ORIGIN:p.web_origin,PUBLIC_RUN_ORIGIN:p.runtime_origin}}};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 const channels=JSON.parse(readFileSync('infra/deploy/channels.json')),state=JSON.parse(readFileSync('infra/production-state.json')),plan=productionPlan(channels,state),out=resolve('target/production-setup');mkdirSync(out,{recursive:true});
 if(!process.argv.includes('--apply')){writeFileSync(resolve(out,'plan.json'),JSON.stringify(plan,null,2)+'\n');console.log(JSON.stringify({plan_only:true,...plan}));}
 else{
  authorizeStorageSetup(state,plan);const token=process.env.CLOUDFLARE_API_TOKEN;if(!token)throw Error('CLOUDFLARE_CREDENTIAL_REQUIRED');
  const manifest=JSON.parse(readFileSync('infra/production-schema.json')),files=readdirSync('database/migrations').filter(x=>x.endsWith('.sql')).sort();if(JSON.stringify(files)!==JSON.stringify(manifest.map(x=>x.file)))throw Error('SCHEMA_REVIEW_REQUIRED');
  for(const m of manifest)if(createHash('sha256').update(readFileSync('database/migrations/'+m.file)).digest('hex')!==m.sha256)throw Error('SCHEMA_REVIEW_REQUIRED');
  const cloud=cloudApi(token,plan.account_id),meter=d1Budget(),receipt={date:new Date().toISOString(),storage_provisioned:false,schemas_applied:false,dns_changed:false,mail_changed:false,workers_deployed:false,deleted_resources:false};
  const save=()=>writeFileSync(resolve(out,'receipt.json'),JSON.stringify({...receipt,d1:meter.usage},null,2)+'\n');
  try{
   const databases=await cloud('/d1/database'),buckets=(await cloud('/r2/buckets')).buckets;
   let db=databases.find(x=>x.name===plan.database_name);if(db&&db.uuid!==channels.production?.database_id)throw Error('EXISTING_DATABASE_REQUIRES_OWNERSHIP_REVIEW');
   if(buckets.some(x=>x.name===plan.bucket)&&channels.production?.bucket!==plan.bucket)throw Error('EXISTING_BUCKET_REQUIRES_OWNERSHIP_REVIEW');
   if(!db)db=await cloud('/d1/database','POST',{name:plan.database_name,primary_location_hint:'weur'});receipt.database_id=db.uuid;save();
   if(db.uuid===channels.staging.database_id)throw Error('SHARED_DATABASE_FORBIDDEN');
   if(!buckets.some(x=>x.name===plan.bucket))await cloud('/r2/buckets','POST',{name:plan.bucket,storageClass:'Standard'});receipt.bucket=plan.bucket;receipt.storage_provisioned=true;save();
   const p={...plan,database_id:db.uuid};channels.production=p;writeFileSync('infra/deploy/channels.json',JSON.stringify(channels,null,2)+'\n');
   const query=async(sql,params=[])=>{meter.before();const r=await cloud('/d1/database/'+db.uuid+'/query','POST',{sql,params});meter.record(r);save();if(r.some(x=>!x.success))throw Error('PRODUCTION_SCHEMA_FAILED');return r};
   await query('CREATE TABLE IF NOT EXISTS d1_migrations(id INTEGER PRIMARY KEY AUTOINCREMENT,name TEXT UNIQUE,applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)');
   const applied=new Set((await query('SELECT name FROM d1_migrations ORDER BY id'))[0].results.map(x=>x.name));
   for(const m of manifest)if(!applied.has(m.file)){await query(readFileSync('database/migrations/'+m.file,'utf8'));await query('INSERT INTO d1_migrations(name) VALUES(?)',[m.file])}
   receipt.schemas_applied=true;const configs=renderProductionConfigs(p);for(const [name,value]of Object.entries(configs))writeFileSync(resolve(out,name+'.json'),JSON.stringify(value,null,2)+'\n');const apiSource=JSON.parse(readFileSync('api/wrangler.jsonc'));apiSource.env.production={name:p.api_worker,workers_dev:false,preview_urls:false,vars:configs.api.vars,d1_databases:configs.api.d1_databases.map(({migrations_dir,...rest})=>({...rest,migrations_dir:'../database/migrations'})),r2_buckets:configs.api.r2_buckets};writeFileSync('api/wrangler.jsonc',JSON.stringify(apiSource,null,2)+'\n');receipt.configurations_rendered=true;save();console.log(JSON.stringify({...receipt,d1:meter.usage}));
  }catch(error){receipt.error=error.message;save();throw error}
 }
}
