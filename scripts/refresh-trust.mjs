// Windows operator refresh of the short-lived signed trust documents (revocation list: 24 h by contract;
// Runtime channel document: 30 days at most). Idempotent: nothing is signed while the live documents have
// plenty of validity left. Private seeds stay under the operator DPAPI slots; nothing is exported.
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
 production:{worker:'kortex-api-production',origin:'https://api.kortex.site',revocationRoot:'infra/trust/revocation-production-root.json',revocationSlot:'revocations-production',releaseRoot:'infra/trust/release-production-root.json',releaseKeys:'infra/trust/release-production-keys.json',releaseSlot:'release-production',channel:'stable',account:'production'},
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

// Revocation list: re-sign the same revocations with a higher version and a fresh 24 h window.
const rootPin=json(profile.revocationRoot),oldRevocations=JSON.parse(binding('REVOCATION_DOCUMENT').text);
if(force||oldRevocations.valid_until-now<REVOCATION_TTL/2){
 const next={...oldRevocations,version:oldRevocations.version+1,issued_at:now-1000,valid_until:now+REVOCATION_TTL-1000,root_key_id:rootPin.id,signature:''},key=privateKey(profile.revocationSlot);
 if(rawKey(key)!==rootPin.public_key_base64)throw Error('REVOCATION_ROOT_MISMATCH');
 next.signature=sign(null,revocationText(next),key).toString('base64');
 if(!verify(null,revocationText(next),createPublicKey({key:Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(rootPin.public_key_base64,'base64')]),format:'der',type:'spki'}),Buffer.from(next.signature,'base64')))throw Error('REVOCATION_SELF_CHECK_FAILED');
 replace.REVOCATION_DOCUMENT=JSON.stringify(next);log('revocation document version '+next.version+' valid until '+new Date(next.valid_until).toISOString());
}else log('revocation document still valid until '+new Date(oldRevocations.valid_until).toISOString());

// Runtime channel document: same channels, next sequence, fresh window (only when under 7 days remain).
const channelsBinding=binding('RUNTIME_CHANNELS');
if(channelsBinding){
 const envelope=JSON.parse(channelsBinding.text),releaseRoot=json(profile.releaseRoot),keys=json(profile.releaseKeys);
 if(force||envelope.document.expires-now<7*86400000){
  const document={...envelope.document,sequence:envelope.document.sequence+1,issued_at:now-1000,expires:Math.min(now+29*86400000,keys.expires),signature:''},key=privateKey(profile.releaseSlot);
  if(document.expires<=now)throw Error('RELEASE_KEYS_EXPIRED_ROTATE_FIRST');
  document.signature=sign(null,runtimeSigningText(document),key).toString('base64');
  const next={document,keys};await verifyRuntime(next,releaseRoot);replace.RUNTIME_CHANNELS=JSON.stringify(next);log('runtime channels sequence '+document.sequence+' valid until '+new Date(document.expires).toISOString());
 }else log('runtime channels still valid until '+new Date(envelope.document.expires).toISOString());
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
