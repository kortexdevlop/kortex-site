// Operator-only certification; the root never enters the ordinary build pipeline.
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {homedir} from 'node:os';
import {resolve} from 'node:path';
import {randomBytes,createPrivateKey,createPublicKey,sign} from 'node:crypto';
import {spawnSync} from 'node:child_process';
if(process.platform!=='win32'||process.env.CI==='true')throw Error('Windows operator required');
const folder=resolve(homedir(),'.kortex-offline'),secret=resolve(folder,'release-staging.dpapi');
function dpapi(value,protect){const method=protect?'Protect':'Unprotect',r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`Add-Type -AssemblyName System.Security; [Convert]::ToBase64String([Security.Cryptography.ProtectedData]::${method}([Convert]::FromBase64String([Console]::In.ReadToEnd().Trim()),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))`],{input:value,encoding:'utf8',windowsHide:true});if(r.status!==0)throw Error('DPAPI unavailable');return r.stdout.trim()}
mkdirSync(folder,{recursive:true});
if(!existsSync(secret)){const seed=randomBytes(32);try{writeFileSync(secret,dpapi(seed.toString('base64'),true),{flag:'wx'})}finally{seed.fill(0)}}
const seed=Buffer.from(dpapi(readFileSync(secret,'utf8'),false),'base64'),rootSeed=Buffer.from(dpapi(readFileSync(resolve(folder,'revocations.dpapi'),'utf8'),false),'base64');
try{
 const privateKey=b=>createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),b]),type:'pkcs8',format:'der'});
 const key=privateKey(seed),rootKey=privateKey(rootSeed),pub=k=>createPublicKey(k).export({format:'der',type:'spki'}).subarray(-32).toString('base64'),root=JSON.parse(readFileSync('infra/trust/revocation-root.json'));
 if(pub(rootKey)!==root.public_key_base64)throw Error('Offline root mismatch');
 const now=Date.now(),k={id:'ci-staging-20261029',public_key_base64:pub(key),not_before:now-60000,expires:now+90*86400000,revoked:false};
 const document={schema_version:1,version:1,issued_at:now-60000,expires:k.expires,root_key_id:root.id,keys:[k],signature:''};
 const text=`kortex-keys-v1\n1\n${document.issued_at}\n${document.expires}\n${root.id}\n${k.id}|${k.public_key_base64}|${k.not_before}|${k.expires}|false\n`;
 document.signature=sign(null,Buffer.from(text),rootKey).toString('base64');
 writeFileSync('infra/trust/release-staging-root.json',JSON.stringify(root,null,2)+'\n');writeFileSync('infra/trust/release-staging-keys.json',JSON.stringify(document,null,2)+'\n');
 console.log('Staging signing key certified; public trust written and private seed kept under operator DPAPI.');
}finally{seed.fill(0);rootSeed.fill(0)}
