// Canonical local builder/sign/verify pipeline; private signing seeds live in memory only.
import {prepareRelease} from './release-tool.mjs';
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {randomBytes,randomUUID,generateKeyPairSync,createPrivateKey,createPublicKey,sign} from 'node:crypto';
const root=resolve(import.meta.dirname,'..'),dir=resolve(root,'target/extension');mkdirSync(dir,{recursive:true});const releases=[];
for(const tool of ['extension-echo-check']){
 const out=resolve(dir,tool);mkdirSync(out,{recursive:true});
 if(existsSync(resolve(out,'publication-request.json'))){releases.push(prepareRelease({tool,out,sourceDir:resolve(root,"tests/integration/sixth-tool-fixture"),dryRun:false}));continue;}
 const seed=randomBytes(32),privateKey=createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'}),offline=generateKeyPairSync('ed25519'),now=Date.now(),id=tool+'-'+randomUUID().slice(0,8);
 const anchor={id:'phase12-root-'+randomUUID().slice(0,8),public_key_base64:offline.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64')};
 const key={id,public_key_base64:createPublicKey(privateKey).export({format:'der',type:'spki'}).subarray(-32).toString('base64'),not_before:now-60000,expires:now+7*86400000,revoked:false};
 const keys={schema_version:1,version:1,issued_at:now-1000,expires:key.expires,root_key_id:anchor.id,keys:[key],signature:''};keys.signature=sign(null,Buffer.from(`kortex-keys-v1\n1\n${keys.issued_at}\n${keys.expires}\n${anchor.id}\n${key.id}|${key.public_key_base64}|${key.not_before}|${key.expires}|false\n`),offline.privateKey).toString('base64');
 const anchorPath=resolve(out,'signing-root.json'),keysPath=resolve(out,'signing-keys.json');writeFileSync(anchorPath,JSON.stringify(anchor));writeFileSync(keysPath,JSON.stringify(keys));
 const old={CI:process.env.CI,KORTEX_SIGNING_KEY_ID:process.env.KORTEX_SIGNING_KEY_ID,KORTEX_SIGNING_KEY_SEED_BASE64:process.env.KORTEX_SIGNING_KEY_SEED_BASE64};
 try{process.env.CI='true';process.env.KORTEX_SIGNING_KEY_ID=id;process.env.KORTEX_SIGNING_KEY_SEED_BASE64=seed.toString('base64');releases.push(prepareRelease({tool,out,sourceDir:resolve(root,"tests/integration/sixth-tool-fixture"),dryRun:false,root:anchorPath,trust:keysPath}));console.log('Signed and verified: '+tool);}
 finally{seed.fill(0);for(const [key,value] of Object.entries(old))if(value===undefined)delete process.env[key];else process.env[key]=value;}
}
writeFileSync(resolve(dir,'releases.json'),JSON.stringify(releases,null,2)+'\n');
