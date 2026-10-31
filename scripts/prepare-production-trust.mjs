import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';import {resolve} from 'node:path';import {homedir} from 'node:os';import {spawnSync} from 'node:child_process';import {pathToFileURL} from 'node:url';
import {randomBytes,createPrivateKey,createPublicKey,createHash,sign,verify} from 'node:crypto';
import {certificateText,runtimeSigningText} from '../shared/typescript/src/runtime.ts';import {revocationText} from '../shared/typescript/src/revocation.ts';
const publicBytes=k=>createPublicKey(k).export({format:'der',type:'spki'}).subarray(-32);
export function productionTrust(rootKey,releaseKey,now=Date.now()){
 const pub=publicBytes(rootKey),root={id:'production-root-'+createHash('sha256').update(pub).digest('hex').slice(0,12),public_key_base64:pub.toString('base64')};
 const key={id:'production-release-1',public_key_base64:publicBytes(releaseKey).toString('base64'),not_before:now-1000,expires:now+90*86400000,revoked:false};
 const keys={schema_version:1,version:1,issued_at:now-1000,expires:key.expires,root_key_id:root.id,keys:[key],signature:''};keys.signature=sign(null,certificateText(keys),rootKey).toString('base64');
 const revocations={schema_version:1,version:1,issued_at:now-1000,valid_until:now+86399000,root_key_id:root.id,revoked_keys:[],revoked_artifacts:[],signature:''};revocations.signature=sign(null,revocationText(revocations),rootKey).toString('base64');
 const document={schema_version:1,sequence:1,issued_at:now-1000,expires:now+7*86400000,channels:{stable:null,staging:null},signing_key_id:key.id,signature:''};document.signature=sign(null,runtimeSigningText(document),releaseKey).toString('base64');
 if(!verify(null,certificateText(keys),createPublicKey(rootKey),Buffer.from(keys.signature,'base64'))||!verify(null,revocationText(revocations),createPublicKey(rootKey),Buffer.from(revocations.signature,'base64')))throw Error('PRODUCTION_TRUST_SELF_CHECK_FAILED');
 return{root,keys,revocations,runtime:{document,keys},host:{root,document:keys,minimum_version:1}};
}
function protectedKey(slot){
 const folder=resolve(homedir(),'.kortex-offline'),file=resolve(folder,slot+'.dpapi');mkdirSync(folder,{recursive:true});
 const dpapi=(value,operation)=>{const r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`Add-Type -AssemblyName System.Security; [Convert]::ToBase64String([Security.Cryptography.ProtectedData]::${operation}([Convert]::FromBase64String([Console]::In.ReadToEnd().Trim()),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))`],{input:value,encoding:'utf8',windowsHide:true});if(r.status!==0)throw Error('PRODUCTION_DPAPI_UNAVAILABLE');return r.stdout.trim()};
 if(!existsSync(file)){const fresh=randomBytes(32);try{writeFileSync(file,dpapi(fresh.toString('base64'),'Protect'),{flag:'wx'})}finally{fresh.fill(0)}}
 const seed=Buffer.from(dpapi(readFileSync(file,'utf8'),'Unprotect'),'base64');try{if(seed.length!==32)throw Error('INVALID_OPERATOR_KEY');return createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'})}finally{seed.fill(0)}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 if(process.platform!=='win32'||process.env.CI==='true')throw Error('WINDOWS_OPERATOR_REQUIRED');
 if(!process.argv.includes('--initialize-production'))throw Error('EXPLICIT_PRODUCTION_INITIALIZATION_REQUIRED');
 const files=['infra/trust/release-production-root.json','infra/trust/release-production-keys.json','infra/trust/revocation-production-root.json','infra/trust/revocations-production.json','infra/trust/runtime-production.json','desktop/assets/production-trust.json'];if(files.some(f=>existsSync(f)))throw Error('PRODUCTION_TRUST_EXISTS_USE_RENEWAL_PROCEDURE');
 const trust=productionTrust(protectedKey('revocations-production'),protectedKey('release-production'));
 const values=[trust.root,trust.keys,trust.root,trust.revocations,trust.runtime,trust.host];for(let i=0;i<files.length;i++)writeFileSync(files[i],JSON.stringify(values[i],null,2)+'\n',{flag:'wx'});
 mkdirSync('docs/evidence',{recursive:true});writeFileSync('docs/evidence/production-trust-preparation.json',JSON.stringify({date:new Date().toISOString(),passed:true,root_id:trust.root.id,release_key_id:trust.keys.keys[0].id,independent_from_staging:true,root_private_key_exported:false,protected_operator_slots:['revocations-production.dpapi','release-production.dpapi'],revocation_valid_until:trust.revocations.valid_until,runtime_published:false},null,2)+'\n');console.log('Production trust prepared and signatures verified; private keys retained under operator DPAPI.');
}
