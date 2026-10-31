// Private production preparation: no public route, preview or workers.dev URL.
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';import {resolve} from 'node:path';import {homedir} from 'node:os';import {spawnSync} from 'node:child_process';import {randomBytes} from 'node:crypto';import {pathToFileURL} from 'node:url';
import {renderProductionConfigs} from './production-setup.mjs';import {cloudApi} from './release-cloud.mjs';
export function privateProductionConfigs(p,trust,site){
 if(!trust.root.id.startsWith('production-root-')||trust.keys.root_key_id!==trust.root.id)throw Error('INDEPENDENT_PRODUCTION_TRUST_REQUIRED');
 const configs=renderProductionConfigs(p),common={workers_dev:false,preview_urls:false,routes:[],send_metrics:false,observability:{enabled:false}};
 const policy={issuer:'https://token.actions.githubusercontent.com',audience:p.api_origin,subject:'repo:kortexdevlop/kortex-site:environment:production',repository:'kortexdevlop/kortex-site',repository_id:'1437766566',ref:'refs/heads/main',workflow_ref:'kortexdevlop/kortex-site/.github/workflows/production-release.yml@refs/heads/main',actions:['publish','revoke','rollback']};
 configs.api={...configs.api,...common,vars:{...configs.api.vars,RUNTIME_ROOT:JSON.stringify(trust.root),RUNTIME_CHANNELS:JSON.stringify(trust.runtime),REVOCATION_ROOT:JSON.stringify(trust.root),REVOCATION_DOCUMENT:JSON.stringify(trust.revocations),ADMIN_AUTH_POLICY:JSON.stringify(policy)}};
 configs.runtime={...configs.runtime,...common,vars:{...configs.runtime.vars,TRUST_KEYS:JSON.stringify(trust.keys.keys),REVOCATION_ROOT:JSON.stringify(trust.root)}};
 configs.website={...site,...configs.website,...common,rules:site.rules,no_bundle:site.no_bundle};return configs;
}
function applicationSecrets(){
 const folder=resolve(homedir(),'.kortex-offline'),file=resolve(folder,'application-production.dpapi');mkdirSync(folder,{recursive:true});
 const transform=(value,method)=>{const r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',`Add-Type -AssemblyName System.Security; [Convert]::ToBase64String([Security.Cryptography.ProtectedData]::${method}([Convert]::FromBase64String([Console]::In.ReadToEnd().Trim()),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))`],{input:value,encoding:'utf8',windowsHide:true});if(r.status!==0)throw Error('PRODUCTION_SECRET_PROTECTION_FAILED');return r.stdout.trim()};
 if(!existsSync(file)){const values=Object.fromEntries(['CURSOR_SIGNING_KEY','SESSION_HMAC_SECRET','PASSWORD_PEPPER'].map(k=>[k,k==='CURSOR_SIGNING_KEY'?randomBytes(32).toString('hex'):randomBytes(32).toString('base64')])),bytes=Buffer.from(JSON.stringify(values));try{writeFileSync(file,transform(bytes.toString('base64'),'Protect'),{flag:'wx'})}finally{bytes.fill(0)}}
 const bytes=Buffer.from(transform(readFileSync(file,'utf8'),'Unprotect'),'base64');try{const values=JSON.parse(bytes.toString('utf8'));for(const k of ['CURSOR_SIGNING_KEY','SESSION_HMAC_SECRET','PASSWORD_PEPPER'])if(typeof values[k]!=='string'||(k==='CURSOR_SIGNING_KEY'?!/^[0-9a-f]{64}$/.test(values[k]):Buffer.from(values[k],'base64').length!==32))throw Error('INVALID_PRODUCTION_SECRET_BUNDLE');return values}finally{bytes.fill(0)}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){
 if(process.platform!=='win32'||process.env.CI==='true'||!process.argv.includes('--prepare-private'))throw Error('EXPLICIT_WINDOWS_OPERATOR_PREPARATION_REQUIRED');
 const state=JSON.parse(readFileSync('infra/production-state.json'));if(state.owner_decisions?.workers_plan_confirmed!=='free'||state.production_storage_provisioned!==true)throw Error('FREE_PRODUCTION_STORAGE_REQUIRED');
 const p=JSON.parse(readFileSync('infra/deploy/channels.json')).production,token=process.env.CLOUDFLARE_API_TOKEN;if(!p||!token)throw Error('PRODUCTION_CONTEXT_REQUIRED');
 const json=f=>JSON.parse(readFileSync(f)),trust={root:json('infra/trust/release-production-root.json'),keys:json('infra/trust/release-production-keys.json'),revocations:json('infra/trust/revocations-production.json'),runtime:json('infra/trust/runtime-production.json')},configs=privateProductionConfigs(p,trust,json('website/dist/server/wrangler.json'));
 const out=resolve('target/production-private');mkdirSync(out,{recursive:true});const secrets=applicationSecrets(),secretValues=Object.values(secrets),cloud=cloudApi(token,p.account_id),receipt={date:new Date().toISOString(),private_preparation:true,public_release:false,dns_changed:false,mail_changed:false,private_services:[],secret_names:Object.keys(secrets),secret_values_exported:false};
 const cli=(args,input)=>{const r=spawnSync(process.execPath,['node_modules/wrangler/bin/wrangler.js',...args],{encoding:'utf8',windowsHide:true,input,timeout:120000,maxBuffer:8*1024*1024,env:{...process.env,CLOUDFLARE_ACCOUNT_ID:p.account_id,CI:'true',WRANGLER_SEND_METRICS:'false'}});let log=(r.stdout??'')+(r.stderr??'');for(const value of [token,...secretValues])log=log.replaceAll(value,'[REDACTED]');writeFileSync(resolve(out,'last-cli.log'),log);if(r.status!==0)throw Error('PRIVATE_PREPARATION_CLI_FAILED_'+args[0]);return log};
 try{
  const custom=await cloud('/workers/domains');if(custom.some(x=>[p.api_worker,p.runtime_worker,p.web_worker].includes(x.service)))throw Error('PRODUCTION_ALREADY_PUBLIC_REQUIRES_RELEASE_PATH');
  const files={};for(const [kind,c]of Object.entries(configs)){files[kind]=resolve(out,kind+'.json');writeFileSync(files[kind],JSON.stringify(c,null,2)+'\n');cli(['deploy','--dry-run','--config',files[kind],'--outdir',resolve(out,kind+'-dry')]);}
  for(const kind of ['api','runtime','website']){
   const worker=configs[kind].name;cli(['deploy','--config',files[kind]]);if(kind==='api')cli(['secret','bulk','--config',files.api],JSON.stringify(secrets));
   const exposure=await cloud('/workers/scripts/'+worker+'/subdomain');if(exposure.enabled||exposure.previews_enabled)throw Error('UNEXPECTED_PUBLIC_WORKER_EXPOSURE');
   const settings=await cloud('/workers/scripts/'+worker+'/settings');if(kind==='api'&&Object.keys(secrets).some(k=>!settings.bindings.some(b=>b.type==='secret_text'&&b.name===k)))throw Error('PRODUCTION_SECRET_BINDING_MISSING');
   receipt.private_services.push({kind,worker,workers_dev_enabled:false,preview_enabled:false});
  }
  receipt.passed=true;console.log(JSON.stringify({passed:true,private_services:receipt.private_services.length,public_release:false}));
 }catch(error){receipt.passed=false;receipt.error=error.message;process.exitCode=1;console.error(error.message)}finally{mkdirSync('docs/evidence',{recursive:true});writeFileSync('docs/evidence/production-private-services.json',JSON.stringify(receipt,null,2)+'\n');for(const k of Object.keys(secrets))secrets[k]=''}
}
