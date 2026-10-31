// Windows operator production release (Workers Free, zero budget). Ordered and resumable:
//   deploy    -> custom domains + Workers from the built artifacts. Build the site first with PUBLIC_KORTEX_TRUST=production
//                (astro build --root website) and the Runtime with scripts/build-runtime.mjs; the site must pin the production root.
//   tools     -> five signed packages into the production D1/R2 through the real publication service,
//                run by the account owner with their own Cloudflare login (the HTTP admin API stays GitHub-OIDC-only)
//   channels  -> signed stable Runtime channel (no unsigned installer in the stable channel)
//   finalize  -> install the signed channels and verify them from the public API
// No DNS record or mail setting is edited here: Cloudflare custom domains create their own records and
// refuse to override existing ones. Private keys stay under the operator DPAPI slots.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,existsSync,rmSync,readdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {homedir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {createHash,createPrivateKey,randomUUID,sign} from 'node:crypto';
import {createRequire} from 'node:module';
import {getPlatformProxy} from 'wrangler';
import {prepareRelease} from './release-tool.mjs';
import {privateProductionConfigs} from './production-private-services.mjs';
import {runtimeSigningText,verifyRuntime} from '../shared/typescript/src/runtime.ts';

const root=resolve(import.meta.dirname,'..'),json=p=>JSON.parse(readFileSync(resolve(root,p),'utf8')),hash=b=>createHash('sha256').update(b).digest('hex');
const phase=process.argv[2];if(!['deploy','tools','channels','finalize'].includes(phase))throw Error('PHASE_REQUIRED');
if(process.platform!=='win32'||process.env.CI==='true')throw Error('WINDOWS_OPERATOR_REQUIRED');
const state=json('infra/production-state.json');
if(state.owner_decisions?.workers_plan_confirmed!=='free'||state.owner_decisions?.monthly_budget_eur!==0||state.production_storage_provisioned!==true)throw Error('FREE_PRODUCTION_STORAGE_REQUIRED');
const p=json('infra/deploy/channels.json').production,out=resolve(root,'target/production-release');mkdirSync(out,{recursive:true});
const trust={root:json('infra/trust/release-production-root.json'),keys:json('infra/trust/release-production-keys.json'),revocations:json('infra/trust/revocations-production.json'),runtime:json('infra/trust/runtime-production.json')};
const receiptPath=resolve(root,'docs/evidence/production-release.json'),receipt=existsSync(receiptPath)?JSON.parse(readFileSync(receiptPath,'utf8')):{schema_version:1,phases:{},mail_records_edited:false,dns_records_edited_directly:false};
const save=()=>writeFileSync(receiptPath,JSON.stringify(receipt,null,2)+'\n');
const dpapi=(value,method)=>{const r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`Add-Type -AssemblyName System.Security; [Convert]::ToBase64String([Security.Cryptography.ProtectedData]::${method}([Convert]::FromBase64String([Console]::In.ReadToEnd().Trim()),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))`],{input:value,encoding:'utf8',windowsHide:true});if(r.status!==0)throw Error('DPAPI_UNAVAILABLE');return r.stdout.trim()};
const releaseSeed=()=>Buffer.from(dpapi(readFileSync(resolve(homedir(),'.kortex-offline','release-production.dpapi'),'utf8'),'Unprotect'),'base64');
function wrangler(args,extra={}){const r=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,...(args[0]==='whoami'?[]:['--profile','kortex'])],{cwd:root,encoding:'utf8',windowsHide:true,timeout:300000,maxBuffer:16*1024*1024,env:{...process.env,CLOUDFLARE_ACCOUNT_ID:p.account_id,CI:'true',WRANGLER_SEND_METRICS:'false',...extra}});if(r.status!==0)throw Error('Wrangler '+args[0]+' failed: '+(r.stderr||r.stdout).slice(-900));return r.stdout}
const identity=JSON.parse(wrangler(['whoami','--json']));assert.ok(identity.accounts.some(a=>a.id===p.account_id),'ACCOUNT_MISMATCH');
const token=JSON.parse(wrangler(['auth','token','--json'])).token;
async function cf(method,path,body){const r=await fetch('https://api.cloudflare.com/client/v4/accounts/'+p.account_id+path,{method,headers:{Authorization:'Bearer '+token,'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:AbortSignal.timeout(30000)});const v=await r.json();if(!r.ok||!v.success)throw Error('Cloudflare '+method+' '+path+' '+r.status+' '+JSON.stringify(v.errors));return v.result}
const host=url=>new URL(url).hostname;
const githubPolicy=JSON.parse(privateProductionConfigs(p,trust,json('website/dist/server/wrangler.json')).api.vars.ADMIN_AUTH_POLICY);
function configs(adminPolicy,runtimeChannels){
 const c=privateProductionConfigs(p,trust,json('website/dist/server/wrangler.json')),route=pattern=>[{pattern,custom_domain:true}];
 c.api.vars={...c.api.vars,ADMIN_AUTH_POLICY:JSON.stringify(adminPolicy),...(runtimeChannels?{RUNTIME_CHANNELS:JSON.stringify(runtimeChannels)}:{})};
 c.api.routes=route(host(p.api_origin));c.runtime.routes=route(host(p.runtime_origin));c.website.routes=route(host(p.web_origin));return c;
}
function write(kind,config){const file=resolve(out,kind+'.json');writeFileSync(file,JSON.stringify(config,null,2)+'\n');return file}
async function health(url,path,ok=r=>r.status===200,attempts=40){let last;for(let i=0;i<attempts;i++){try{const r=await fetch(url+path,{cache:'no-store',redirect:'manual',signal:AbortSignal.timeout(15000)});if(ok(r))return r;last=r.status}catch(e){last=e.message}await new Promise(r=>setTimeout(r,3000))}throw Error('NOT_READY '+url+path+' '+last)}

try{
 if(phase==='deploy'){
  const viewer=readdirSync(resolve(root,'website/dist/client/_astro')).filter(f=>f.startsWith('ToolViewer')&&f.endsWith('.js')).map(f=>readFileSync(resolve(root,'website/dist/client/_astro',f),'utf8'));
  if(viewer.length!==1||!viewer[0].includes(trust.root.id)||viewer[0].includes('release-staging'))throw Error('SITE_MUST_BE_BUILT_WITH_PRODUCTION_TRUST');
  const channelsFile=resolve(out,'channels.json'),c=configs(githubPolicy,existsSync(channelsFile)?JSON.parse(readFileSync(channelsFile,'utf8')):undefined);
  for(const kind of ['api','runtime','website']){wrangler(['deploy','--config',write(kind,c[kind])]);console.log('deployed '+kind)}
  for(const origin of [p.api_origin,p.runtime_origin,p.web_origin]){const r=await health(origin,'/health.json',r=>r.status===200||r.status===404,60);console.log(origin,r.status)}
  await health(p.api_origin,'/api/v1/health');
  receipt.phases.deploy={date:new Date().toISOString(),origins:[p.api_origin,p.runtime_origin,p.web_origin],admin_policy:'GitHub OIDC only (production API refuses operator issuers)'};
 }
 if(phase==='tools'){
  const seed=releaseSeed(),tools=['calculatrice','convertisseur-casse','calculateur-hash','timestamp-utc','explorateur-csv'],releases=[];
  process.env.CI='true';process.env.KORTEX_SIGNING_KEY_ID=trust.keys.keys[0].id;process.env.KORTEX_SIGNING_KEY_SEED_BASE64=seed.toString('base64');
  try{for(const tool of tools)releases.push(prepareRelease({tool,out:resolve(out,'tools',tool),root:resolve(root,'infra/trust/release-production-root.json'),trust:resolve(root,'infra/trust/release-production-keys.json')}))}
  finally{seed.fill(0);delete process.env.KORTEX_SIGNING_KEY_SEED_BASE64;delete process.env.KORTEX_SIGNING_KEY_ID;delete process.env.CI}
  const previousToken=process.env.CLOUDFLARE_API_TOKEN;process.env.CLOUDFLARE_API_TOKEN=token;process.env.CLOUDFLARE_ACCOUNT_ID=p.account_id;
  const proxyConfig=configs(githubPolicy).api;proxyConfig.d1_databases=proxyConfig.d1_databases.map(d=>({...d,remote:true}));proxyConfig.r2_buckets=proxyConfig.r2_buckets.map(b=>({...b,remote:true}));const configFile=write('api-proxy',proxyConfig);let platform;const published=[];const {buildSync}=createRequire(resolve(root,'website/package.json'))('esbuild'),bundle=resolve(out,'publication-service.mjs');buildSync({entryPoints:[resolve(root,'api/src/services/publication.ts')],bundle:true,format:'esm',platform:'neutral',outfile:bundle,logLevel:'error'});if(!globalThis.crypto.DigestStream)Object.defineProperty(globalThis.crypto,'DigestStream',{configurable:true,value:class{constructor(algorithm){const chunks=[];this.writable=new WritableStream({write(chunk){chunks.push(Buffer.from(chunk))}});this.digest=new Promise((done,fail)=>{this.writable.closed?.then?.(()=>{}).catch?.(()=>{});this._done=()=>crypto.subtle.digest(algorithm,Buffer.concat(chunks)).then(done,fail)})}getWriter(){const writer=this.writable.getWriter(),close=writer.close.bind(writer);writer.close=async()=>{await close();this._done()};return writer}}});const {publish:publishService}=await import(pathToFileURL(bundle).href);
  try{
   platform=await getPlatformProxy({configPath:configFile,remoteBindings:true,persist:false,envFiles:[]});
   const key=trust.keys.keys[0];
   await platform.env.DB.prepare('INSERT INTO signing_keys(id,public_key_base64,trust_version,not_before,expires) SELECT ?,?,1,?,? WHERE NOT EXISTS(SELECT 1 FROM signing_keys WHERE id=?)').bind(key.id,key.public_key_base64,key.not_before,key.expires,key.id).run();
   assert.equal(await platform.env.DB.prepare('SELECT public_key_base64 FROM signing_keys WHERE id=?').bind(key.id).first('public_key_base64'),key.public_key_base64);
   for(const release of releases){
    const request=JSON.parse(readFileSync(release.publication_request,'utf8')),bytes=readFileSync(release.archive);assert.equal(hash(bytes),request.descriptor.sha256);assert.equal(request.descriptor.signing_key_id,key.id);
    const put=await platform.env.TOOLS_BUCKET.put(request.r2_key,bytes,{onlyIf:{etagDoesNotMatch:'*'},httpMetadata:{contentType:'application/vnd.kortex.tool+zip'}});
    if(!put){const existing=await platform.env.TOOLS_BUCKET.get(request.r2_key);assert.equal(hash(Buffer.from(await existing.arrayBuffer())),request.descriptor.sha256)}
    const exists=await fetch(p.api_origin+'/api/v1/tools/'+release.tool_id);
    if(exists.status===404)await publishService(platform.env,request,{subject:'production-operator-direct',jti:randomUUID()});
    const detail=await fetch(p.api_origin+'/api/v1/tools/'+release.tool_id);assert.equal(detail.status,200);assert.equal((await detail.json()).descriptor.sha256,release.sha256);
    const download=await fetch(p.api_origin+'/api/v1/downloads/'+release.tool_id+'/'+request.manifest.version);assert.equal(download.status,200);assert.equal(hash(Buffer.from(await download.arrayBuffer())),release.sha256);
    published.push({id:release.tool_id,version:request.manifest.version,sha256:release.sha256});console.log('published '+release.tool_id)
   }
  }finally{if(platform)await platform.dispose();if(previousToken===undefined)delete process.env.CLOUDFLARE_API_TOKEN;else process.env.CLOUDFLARE_API_TOKEN=previousToken}
  receipt.phases.tools={date:new Date().toISOString(),tools:published,signing_key_id:trust.keys.keys[0].id};
 }
 if(phase==='channels'){
  const version=await (await health(p.runtime_origin,'/version.json')).json(),keys=trust.keys,now=Date.now();
  assert.ok(/^[a-f0-9]{64}$/.test(version.bundle_sha256)&&version.script_integrity);
  const document={schema_version:1,sequence:trust.runtime.document.sequence+1,issued_at:now-1000,expires:Math.min(now+29*86400000,keys.expires),channels:{stable:{runtime_version:version.runtime_version,abi_versions:version.abi_versions,protocol_versions:version.protocol_versions,ui_schema_versions:version.ui_schema_versions,web:{origin:p.runtime_origin,version_url:p.runtime_origin+'/version.json',bundle_sha256:version.bundle_sha256,script_integrity:version.script_integrity},desktop:null},staging:null},signing_key_id:keys.keys[0].id,signature:''};
  const seed=releaseSeed();try{const key=createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'});document.signature=sign(null,runtimeSigningText(document),key).toString('base64')}finally{seed.fill(0)}
  const envelope={document,keys};await verifyRuntime(envelope,trust.root);writeFileSync(resolve(out,'channels.json'),JSON.stringify(envelope,null,2)+'\n');
  receipt.phases.channels={date:new Date().toISOString(),sequence:document.sequence,expires:document.expires,runtime_version:version.runtime_version,bundle_sha256:version.bundle_sha256,desktop_in_stable_channel:false};
 }
 if(phase==='finalize'){
  const envelope=JSON.parse(readFileSync(resolve(out,'channels.json'),'utf8')),c=configs(githubPolicy,envelope);
  wrangler(['deploy','--config',write('api',c.api)]);
  const live=await (await health(p.api_origin,'/api/v1/runtime/channels')).json();assert.equal(live.document.sequence,envelope.document.sequence);await verifyRuntime(live,trust.root);
  receipt.phases.finalize={date:new Date().toISOString(),admin_authority:'github-oidc-only',channel_sequence:live.document.sequence};receipt.passed=true;
 }
}catch(error){receipt.phases[phase+'_error']=String(error.message).slice(0,1200);save();console.error(error.message);process.exitCode=1;throw error}
save();console.log(JSON.stringify({phase,passed:true}));
