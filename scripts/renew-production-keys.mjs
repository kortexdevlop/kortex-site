// Windows operator rotation of the production delegated key. D1 keeps signing keys immutable, so a new key takes
// over signing while the offline root re-certifies the previous one for longer: published packages stay valid
// without being republished. Private seeds stay under DPAPI, and the new D1 row is added afterwards.
//   node scripts/renew-production-keys.mjs [--days 730]
import {readFileSync,writeFileSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {homedir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {createPrivateKey,createPublicKey,randomBytes,sign,verify} from 'node:crypto';
import {certificateText} from '../shared/typescript/src/runtime.ts';

const root=resolve(import.meta.dirname,'..'),json=p=>JSON.parse(readFileSync(resolve(root,p),'utf8'));
if(process.platform!=='win32'||process.env.CI==='true')throw Error('WINDOWS_OPERATOR_REQUIRED');
const at=process.argv.indexOf('--days'),days=at>=0?Number(process.argv[at+1]):730;if(!Number.isInteger(days)||days<30||days>1095)throw Error('INVALID_DURATION');
const dpapi=(value,method)=>{const r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`Add-Type -AssemblyName System.Security; [Convert]::ToBase64String([Security.Cryptography.ProtectedData]::${method}([Convert]::FromBase64String([Console]::In.ReadToEnd().Trim()),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))`],{input:value,encoding:'utf8',windowsHide:true});if(r.status!==0)throw Error('DPAPI_UNAVAILABLE');return r.stdout.trim()};
const privateKey=seed=>createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'}),publicRaw=key=>createPublicKey(key).export({format:'der',type:'spki'}).subarray(-32).toString('base64');
const folder=resolve(homedir(),'.kortex-offline'),rootSeed=Buffer.from(dpapi(readFileSync(resolve(folder,'revocations-production.dpapi'),'utf8'),'Unprotect'),'base64');let rootKey;
try{rootKey=privateKey(rootSeed)}finally{rootSeed.fill(0)}
const pin=json('infra/trust/release-production-root.json'),old=json('infra/trust/release-production-keys.json');if(publicRaw(rootKey)!==pin.public_key_base64)throw Error('ROOT_MISMATCH');
const next=Math.max(...old.keys.map(k=>Number(k.id.split('-').pop())))+1,id='production-release-'+next,slot=resolve(folder,'release-production-'+next+'.dpapi');if(existsSync(slot))throw Error('KEY_SLOT_EXISTS');
const seed=randomBytes(32);let publicKey;try{publicKey=publicRaw(privateKey(seed));writeFileSync(slot,dpapi(seed.toString('base64'),'Protect'),{flag:'wx'})}finally{seed.fill(0)}
const now=Date.now(),expires=now+days*86400000;if(expires<=old.expires)throw Error('RENEWAL_MUST_EXTEND');
const keys={...old,version:old.version+1,issued_at:now-1000,expires,keys:[...old.keys.map(k=>k.revoked?k:{...k,expires}),{id,public_key_base64:publicKey,not_before:now-1000,expires,revoked:false}],signature:''};
keys.signature=sign(null,certificateText(keys),rootKey).toString('base64');
if(!verify(null,certificateText(keys),createPublicKey(rootKey),Buffer.from(keys.signature,'base64')))throw Error('CERTIFICATE_SELF_CHECK_FAILED');
const write=(file,value)=>writeFileSync(resolve(root,file),JSON.stringify(value,null,2)+'\n');
write('infra/trust/release-production-keys.json',keys);
write('infra/trust/runtime-production.json',{...json('infra/trust/runtime-production.json'),keys});
write('desktop/assets/production-trust.json',{...json('desktop/assets/production-trust.json'),document:keys});
console.log(JSON.stringify({version:keys.version,active:id,expires:new Date(expires).toISOString(),keys:keys.keys.map(k=>k.id),sql:`INSERT INTO signing_keys(id,public_key_base64,trust_version,not_before,expires) VALUES('${id}','${publicKey}',${keys.version},${now-1000},${expires})`}));
