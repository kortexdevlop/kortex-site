// Windows operator refresh of the short-lived signed trust documents (revocation list: 24 h by contract;
// Runtime channel document: 30 days at most). Private seeds stay under the operator DPAPI slots.
// Production pre-signs 90 days of documents into the private bucket, so the service keeps running with
// this machine switched off; the API only selects the one valid now. Staging renews its live documents.
// Run with --force after a revocation or a Runtime release: it replaces the whole schedule.
//   node scripts/refresh-trust.mjs --env production|staging [--force]
import {readFileSync,writeFileSync,mkdirSync,appendFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {homedir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {createPrivateKey,createPublicKey,sign,verify} from 'node:crypto';
import {revocationText,REVOCATION_TTL} from '../shared/typescript/src/revocation.ts';
import {runtimeSigningText,verifyRuntime} from '../shared/typescript/src/runtime.ts';

const root=resolve(import.meta.dirname,'..'),json=p=>JSON.parse(readFileSync(resolve(root,p),'utf8'));
const env=process.argv[process.argv.indexOf('--env')+1],force=process.argv.includes('--force');
const profiles={
 production:{worker:'kortex-api-production',origin:'https://api.kortex.site',revocationRoot:'infra/trust/revocation-production-root.json',revocationSlot:'revocations-production',releaseRoot:'infra/trust/release-production-root.json',releaseKeys:'infra/trust/release-production-keys.json',releaseSlot:'release-production',channel:'stable',account:'production',bucket:'kortex-tools-production'},
 staging:{worker:'kortex-api-staging',origin:'https://kortex-api-staging.userpgl.workers.dev',revocationRoot:'infra/trust/revocation-root.json',revocationSlot:'revocations',releaseRoot:'infra/trust/release-staging-root.json',releaseKeys:'infra/trust/release-staging-keys.json',releaseSlot:'release-staging',channel:'staging',account:'staging'}
};
const profile=profiles[env];if(!profile||process.platform!=='win32'||process.env.CI==='true')throw Error('WINDOWS_OPERATOR_AND_ENV_REQUIRED');
const account=json('infra/deploy/channels.json')[profile.account].account_id;
mkdirSync(resolve(root,'target'),{recursive:true});const log=line=>{const text=new Date().toISOString()+' '+env+' '+line;console.log(text);appendFileSync(resolve(root,'target/trust-refresh.log'),text+'\n')};
const dpapi=(value,method)=>{const r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`Add-Type -AssemblyName System.Security; [Convert]::ToBase64String([Security.Cryptography.ProtectedData]::${method}([Convert]::FromBase64String([Console]::In.ReadToEnd().Trim()),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))`],{input:value,encoding:'utf8',windowsHide:true});if(r.status!==0)throw Error('DPAPI_UNAVAILABLE');return r.stdout.trim()};
const privateKey=slot=>{const seed=Buffer.from(dpapi(readFileSync(resolve(homedir(),'.kortex-offline',slot+'.dpapi'),'utf8'),'Unprotect'),'base64');try{return createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'})}finally{seed.fill(0)}};
const rawKey=key=>createPublicKey(key).export({format:'der',type:'spki'}).subarray(-32).toString('base64');
const wrangler=args=>{const r=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,'--profile','kortex'],{cwd:root,encoding:'utf8',windowsHide:true,timeout:120000,env:{...process.env,CI:'true',CLOUDFLARE_ACCOUNT_ID:account,WRANGLER_SEND_METRICS:'false'}});if(r.status!==0)throw Error('WRANGLER_'+args[0]+'_FAILED');return r.stdout};
const token=JSON.parse(wrangler(['auth','token','--json'])).token,base='https://api.cloudflare.com/client/v4/accounts/'+account+'/workers/scripts/'+profile.worker+'/settings';
const settings=async()=>{const r=await fetch(base,{headers:{Authorization:'Bearer '+token},signal:AbortSignal.timeout(30000)}),v=await r.json();if(!r.ok||!v.success)throw Error('SETTINGS_READ_FAILED');return v.result};
const live=await settings(),binding=name=>live.bindings.find(b=>b.name===name&&b.type==='plain_text');
const replace={},now=Date.now();

const rootPin=json(profile.revocationRoot),oldRevocations=JSON.parse(binding('REVOCATION_DOCUMENT').text),signRevocation=(document,key)=>({...document,signature:sign(null,revocationText(document),key).toString('base64')});
const revocationKey=()=>{const key=privateKey(profile.revocationSlot);if(rawKey(key)!==rootPin.public_key_base64)throw Error('REVOCATION_ROOT_MISMATCH');return key};
const channelsBinding=binding('RUNTIME_CHANNELS'),releaseRoot=json(profile.releaseRoot),keys=json(profile.releaseKeys),DAY=86400000;
const signChannels=(base,sequence,issued,key)=>{const document={...base,sequence,issued_at:issued,expires:Math.min(issued+29*DAY,keys.expires),signature:''};document.signature=sign(null,runtimeSigningText(document),key).toString('base64');return{document,keys}};
if(profile.bucket){
 const scheduleFile=resolve(root,'target/trust-schedule-'+env+'.json');let saved=null;try{saved=json(scheduleFile)}catch{}
 if(keys.expires-now<45*DAY)log('WARNING release key certificate expires '+new Date(keys.expires).toISOString()+': renew it before the schedule runs out');
 if(!force&&saved&&saved.horizon-now>=45*DAY)log('schedule valid until '+new Date(saved.horizon).toISOString());
 else{
  // Versions embed the 12 h slot and a rebuild counter, so they only ever grow for every client.
  const SLOT=REVOCATION_TTL/2,slot=Math.floor(now/SLOT),generation=Math.max(saved?.generation??0,oldRevocations.version>=1e6?oldRevocations.version%1000:0)+1;if(generation>=1000)throw Error('SCHEDULE_GENERATION_EXHAUSTED');
  const key=revocationKey(),revocations=Array.from({length:180},(_,i)=>signRevocation({...oldRevocations,version:(slot+i)*1000+generation,issued_at:(slot+i)*SLOT,valid_until:(slot+i)*SLOT+REVOCATION_TTL,root_key_id:rootPin.id,signature:''},key));
  for(const document of [revocations[0],revocations.at(-1)])if(!verify(null,revocationText(document),createPublicKey({key:Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(rootPin.public_key_base64,'base64')]),format:'der',type:'spki'}),Buffer.from(document.signature,'base64')))throw Error('REVOCATION_SELF_CHECK_FAILED');
  const envelope=JSON.parse(channelsBinding.text),served=await(await fetch(profile.origin+'/api/v1/runtime/channels',{cache:'no-store',signal:AbortSignal.timeout(15000)})).json().catch(()=>null),sequence=Math.max(envelope.document.sequence,served?.document?.sequence??0),release=privateKey(profile.releaseSlot),channels=[];
  for(let i=0;i<7&&now-1000+i*14*DAY<keys.expires-DAY;i++)channels.push(signChannels(envelope.document,sequence+1+i,now-1000+i*14*DAY,release));
  await verifyRuntime(channels[0],releaseRoot);
  const horizon=Math.min(revocations.at(-1).valid_until,channels.at(-1).document.expires),file=resolve(root,'target/trust-schedule-upload.json');writeFileSync(file,JSON.stringify({revocations,channels}));
  wrangler(['r2','object','put',profile.bucket+'/trust/schedule.json','--file',file,'--content-type','application/json','--remote']);
  writeFileSync(scheduleFile,JSON.stringify({horizon,generation})+'\n');
  replace.REVOCATION_DOCUMENT=JSON.stringify(revocations[0]);replace.RUNTIME_CHANNELS=JSON.stringify(channels[0]);log('schedule rebuilt, generation '+generation+', valid until '+new Date(horizon).toISOString());
 }
}else{
 // Revocation list: re-sign the same revocations with a higher version and a fresh 24 h window.
 if(force||oldRevocations.valid_until-now<REVOCATION_TTL/2){
  const next=signRevocation({...oldRevocations,version:oldRevocations.version+1,issued_at:now-1000,valid_until:now+REVOCATION_TTL-1000,root_key_id:rootPin.id,signature:''},revocationKey());
  replace.REVOCATION_DOCUMENT=JSON.stringify(next);log('revocation document version '+next.version+' valid until '+new Date(next.valid_until).toISOString());
 }else log('revocation document still valid until '+new Date(oldRevocations.valid_until).toISOString());
 // Runtime channel document: same channels, next sequence, fresh window (only when under 7 days remain).
 if(channelsBinding){
  const envelope=JSON.parse(channelsBinding.text);
  if(force||envelope.document.expires-now<7*DAY){
   if(Math.min(now+29*DAY,keys.expires)<=now)throw Error('RELEASE_KEYS_EXPIRED_ROTATE_FIRST');
   const next=signChannels(envelope.document,envelope.document.sequence+1,now-1000,privateKey(profile.releaseSlot));await verifyRuntime(next,releaseRoot);replace.RUNTIME_CHANNELS=JSON.stringify(next);log('runtime channels sequence '+next.document.sequence+' valid until '+new Date(next.document.expires).toISOString());
  }else log('runtime channels still valid until '+new Date(envelope.document.expires).toISOString());
 }
}

if(Object.keys(replace).length){
 const bindings=live.bindings.map(b=>b.type==='plain_text'&&replace[b.name]?{...b,text:replace[b.name]}:b),form=new FormData();form.set('settings',JSON.stringify({bindings}));
 const r=await fetch(base,{method:'PATCH',headers:{Authorization:'Bearer '+token},body:form,redirect:'error',signal:AbortSignal.timeout(30000)});if(!r.ok||!(await r.json()).success)throw Error('SETTINGS_UPDATE_FAILED');
 if(replace.REVOCATION_DOCUMENT)writeFileSync(resolve(root,'target/revocations-'+env+'-latest.json'),replace.REVOCATION_DOCUMENT+'\n');
 if(replace.RUNTIME_CHANNELS)writeFileSync(resolve(root,'target/channels-'+env+'-latest.json'),replace.RUNTIME_CHANNELS+'\n');
}
// Propagation check from the public edge.
let ok=false;for(let i=0;i<20&&!ok;i++){try{const r=await fetch(profile.origin+'/api/v1/trust/revocations',{cache:'no-store',signal:AbortSignal.timeout(15000)});ok=r.status===200}catch{}if(!ok)await new Promise(r=>setTimeout(r,3000))}
if(!ok){log('FAILED public revocation endpoint not available');process.exitCode=1}else log('public revocation endpoint OK');
