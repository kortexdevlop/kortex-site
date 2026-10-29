import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {createPrivateKey,sign} from 'node:crypto';
import {pathToFileURL} from 'node:url';
import {runtimeSigningText,runtimeCompatible,verifyRuntime} from '../shared/typescript/src/runtime.ts';
import {channels,context,protection} from './ci-publish-tool.mjs';
export async function publishRuntime(env=process.env,channel='staging',{operator=false}={}){
 if(operator&&(process.platform!=='win32'||env.CI==='true'||channel!=='staging'))throw Error('LOCAL_STAGING_OPERATOR_REQUIRED');
 const c=operator?channels.staging:context(env,channel);if(!operator)await protection(env,channel);if(channel!=='staging')throw Error('PRODUCTION_RELEASE_GATE');
 if(!env.CLOUDFLARE_API_TOKEN||!env.KORTEX_SIGNING_KEY_SEED_BASE64)throw Error('PROTECTED_CREDENTIAL_REQUIRED');
 const request=async(path)=>{const r=await fetch(`https://api.cloudflare.com/client/v4/accounts/${c.account_id}${path}`,{headers:{Authorization:'Bearer '+env.CLOUDFLARE_API_TOKEN},redirect:'error',signal:AbortSignal.timeout(30000)});const v=await r.json();if(!r.ok||!v.success)throw Error('CLOUDFLARE_UNAVAILABLE');return v.result};
 const live=await request('/workers/scripts/'+c.api_worker+'/settings'),vars=Object.fromEntries(live.bindings.filter(b=>b.type==='plain_text').map(b=>[b.name,b.text]));
 const root=JSON.parse(readFileSync('infra/trust/release-staging-root.json')),keys=JSON.parse(readFileSync('infra/trust/release-staging-keys.json'));
 const response=await fetch(c.runtime_origin+'/version.json',{cache:'no-store',redirect:'error',signal:AbortSignal.timeout(15000)});if(!response.ok)throw Error('RUNTIME_NOT_DEPLOYED');const version=await response.json();
 const previous=vars.RUNTIME_CHANNELS?JSON.parse(vars.RUNTIME_CHANNELS):null;if(previous)await verifyRuntime(previous,root,1,previous.document.issued_at+1);
 const local=env.KORTEX_RUNTIME_EXISTING_ONLY==='true'?previous?.document.channels.staging?.web:JSON.parse(readFileSync('target/web-dist/version.json'));if(!local||version.bundle_sha256!==local.bundle_sha256||env.KORTEX_RUNTIME_EXISTING_ONLY!=='true'&&version.runtime_version!==local.runtime_version||!version.script_integrity)throw Error('RUNTIME_BUNDLE_MISMATCH');
 const old=previous?.document.channels.staging;if(old&&!runtimeCompatible(old.runtime_version,'1.0',version.runtime_version))throw Error('RUNTIME_DOWNGRADE_REQUIRES_OPERATOR_ROLLBACK');
 const now=Date.now(),document={schema_version:1,sequence:(previous?.document.sequence??0)+1,issued_at:now-1000,expires:Math.min(now+7*86400000,keys.expires),channels:{stable:previous?.document.channels.stable??null,staging:{runtime_version:version.runtime_version,abi_versions:version.abi_versions,protocol_versions:version.protocol_versions,ui_schema_versions:version.ui_schema_versions,web:{origin:c.runtime_origin,version_url:c.runtime_origin+'/version.json',bundle_sha256:version.bundle_sha256,script_integrity:version.script_integrity},desktop:old?.desktop??null}},signing_key_id:env.KORTEX_SIGNING_KEY_ID,signature:''};
 if(env.KORTEX_DESKTOP_DESCRIPTOR){const d=JSON.parse(readFileSync(env.KORTEX_DESKTOP_DESCRIPTOR));document.channels.staging.desktop={version:d.version,platform:'windows-x64',url:d.url,sha256:d.sha256,code_signed:d.code_signed}}
 const seed=Buffer.from(env.KORTEX_SIGNING_KEY_SEED_BASE64,'base64');try{if(seed.length!==32)throw Error('INVALID_SIGNING_KEY');const key=createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'});document.signature=sign(null,runtimeSigningText(document),key).toString('base64')}finally{seed.fill(0)}
 const envelope={document,keys};await verifyRuntime(envelope,root);
 const bindings=live.bindings.filter(b=>!['RUNTIME_ROOT','RUNTIME_CHANNELS'].includes(b.name));bindings.push({type:'plain_text',name:'RUNTIME_ROOT',text:JSON.stringify(root)},{type:'plain_text',name:'RUNTIME_CHANNELS',text:JSON.stringify(envelope)});
 const body=new FormData();body.set('settings',JSON.stringify({bindings}));const patched=await fetch(`https://api.cloudflare.com/client/v4/accounts/${c.account_id}/workers/scripts/${c.api_worker}/settings`,{method:'PATCH',headers:{Authorization:'Bearer '+env.CLOUDFLARE_API_TOKEN},body,redirect:'error',signal:AbortSignal.timeout(30000)});if(!patched.ok||!(await patched.json()).success)throw Error('RUNTIME_CHANNEL_PUBLICATION_FAILED');
 const visible=await fetch(c.api_origin+'/api/v1/runtime/channels',{cache:'no-store',signal:AbortSignal.timeout(15000)});if(!visible.ok)throw Error('RUNTIME_CHANNEL_NOT_VISIBLE');const checked=await verifyRuntime(await visible.json(),root,document.sequence);if(checked.signature!==document.signature)throw Error('RUNTIME_CHANNEL_MISMATCH');
 mkdirSync('target/ci-reports/runtime',{recursive:true});writeFileSync('target/ci-reports/runtime/channels.json',JSON.stringify(envelope,null,2)+'\n');return{published:true,channel,sequence:document.sequence,runtime_version:version.runtime_version,bundle_sha256:version.bundle_sha256};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){try{console.log(JSON.stringify(await publishRuntime(process.env,'staging',{operator:process.argv.includes('--operator')})))}catch(e){console.error(e.message);process.exitCode=1}}
