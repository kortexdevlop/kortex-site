// Controlled staging only. OAuth and signing private keys remain in process memory.
import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,readdirSync} from 'node:fs';
import {resolve,relative} from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash,generateKeyPairSync,randomUUID,sign} from 'node:crypto';
import {getPlatformProxy} from 'wrangler';
import {chromium,webkit} from 'playwright';
import {trustedFiles,storedZip} from './staging-fixtures.mjs';
const root=resolve(import.meta.dirname,'..'),dir=resolve(root,'target/web-hot-publish');mkdirSync(dir,{recursive:true});
const json=path=>JSON.parse(readFileSync(path,'utf8')),hash=b=>createHash('sha256').update(b).digest('hex');
function cli(args){const r=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,...(args[0]==='whoami'?[]:['--profile','kortex'])],{cwd:root,encoding:'utf8',windowsHide:true,timeout:180000,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'}});if(r.status!==0)throw Error('Wrangler '+args.slice(0,3).join(' ')+' failed: '+r.stderr.slice(-700));return r.stdout;}
const ownership=json(resolve(root,'target/cloud-staging-resources.json')),identity=JSON.parse(cli(['whoami','--json']));assert.equal(identity.accounts.length,1);assert.equal(identity.accounts[0].id,ownership.account);
const token=JSON.parse(cli(['auth','token','--json'])).token,account=ownership.account;
async function cf(method,path,body){const response=await fetch('https://api.cloudflare.com/client/v4'+path,{method,headers:{Authorization:'Bearer '+token,'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:AbortSignal.timeout(30000)});const data=await response.json();if(!response.ok||data.success===false)throw Error('Cloudflare '+response.status+' '+JSON.stringify(data.errors||[]));return data.result;}
const report={date:new Date().toISOString(),staging_only:true,production_touched:false,mail_touched:false,ci:'controlled local staging operator; GitHub OIDC not claimed',phases:[],passed:false};
const proof=resolve(root,'docs/evidence/web-hot-publish.json');function phase(name,details={}){report.phases.push({name,...details});writeFileSync(proof,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({phase:name,...details}));}
const sourceConfig=json(resolve(root,'api/wrangler.jsonc')),base=sourceConfig.env.staging;assert.equal(base.name,ownership.worker);assert.equal(base.d1_databases[0].database_id,ownership.database_id);assert.equal(base.r2_buckets[0].bucket_name,ownership.bucket);
const sub=await cf('GET',`/accounts/${account}/workers/subdomain`);assert.equal(sub.subdomain,'userpgl');
const web='https://kortex-web-staging.userpgl.workers.dev',run='https://kortex-run-staging.userpgl.workers.dev',api='https://kortex-api-staging.userpgl.workers.dev';Object.assign(report,{web_origin:web,run_origin:run,api_origin:api});
const workers=await cf('GET',`/accounts/${account}/workers/scripts`);
for(const name of ['kortex-web-staging','kortex-run-staging'])if(workers.some(w=>w.id===name)){
 const priorPath=resolve(dir,'ownership.json');let prior;try{prior=json(priorPath);}catch{}if(prior?.account!==account||!prior.workers?.includes(name))throw Error('Unowned staging Worker '+name);
}
writeFileSync(resolve(dir,'ownership.json'),JSON.stringify({account,workers:['kortex-web-staging','kortex-run-staging']}));
const operator=generateKeyPairSync('ed25519'),kid='hot-operator-'+randomUUID().slice(0,8),policy={issuer:'urn:kortex:staging-operator',audience:api,subject:'kortex-staging-operator',repository:'local/kortex-staging',repository_id:'0',ref:'refs/heads/staging',workflow_ref:'local/kortex-staging/staging-publication@refs/heads/staging',actions:['publish','revoke','rollback'],jwks:{keys:[{...operator.publicKey.export({format:'jwk'}),kid,alg:'EdDSA',use:'sig'}]}};
const apiConfig={...base,main:resolve(root,'api/src/index.ts'),compatibility_date:sourceConfig.compatibility_date,send_metrics:false,vars:{...base.vars,PUBLIC_WEB_ORIGIN:web,ADMIN_AUTH_POLICY:JSON.stringify(policy)},d1_databases:base.d1_databases.map(d=>({...d,migrations_dir:resolve(root,'database/migrations')}))};
const apiPath=resolve(dir,'api.json');writeFileSync(apiPath,JSON.stringify(apiConfig));
const release=generateKeyPairSync('ed25519'),keyId='hot-release-'+randomUUID().slice(0,8),now=Date.now(),releaseKey={id:keyId,public_key_base64:release.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64'),not_before:now-60000,expires:now+86400000,revoked:false};
const tool='hot-publication-'+randomUUID().slice(0,8),files=trustedFiles(readFileSync(resolve(root,'target/staging-cloud/downloaded-calculatrice.tool'))),manifest=JSON.parse(files.get('manifest.json')),contract=JSON.parse(files.get('contract.json'));
Object.assign(manifest,{id:tool,name:'Calculatrice de validation à chaud',version:'1.0.0'});manifest.seo={title:'Calculatrice de validation à chaud — Kortex',description:manifest.summary};contract.tool_id=tool;files.set('manifest.json',Buffer.from(JSON.stringify(manifest)));files.set('contract.json',Buffer.from(JSON.stringify(contract)));
const bytes=storedZip(files),sha256=hash(bytes),descriptor={schema_version:1,tool_id:tool,version:'1.0.0',abi_version:'1.0',sha256,signing_key_id:keyId,signature:sign(null,Buffer.from(`kortex-artifact-v1\n${tool}\n1.0.0\n1.0\n${sha256}\n`),release.privateKey).toString('base64')};
const published={request_id:randomUUID(),descriptor,manifest,contract,r2_key:`tools/${tool}/1.0.0/${sha256}.tool`};
const existingKeys=json(resolve(root,'target/staging-cloud/host-trust.json')).document.keys.filter(k=>!k.revoked&&k.expires>Date.now());
const compiled=resolve(root,'website/dist/server'),original=json(resolve(compiled,'wrangler.json'));
assert.equal(original.kv_namespaces.length,0,'No unused session KV is needed');
const siteConfig={...original,name:'kortex-web-staging',topLevelName:'kortex-web-staging',configPath:resolve(dir,'site.json'),userConfigPath:resolve(dir,'site.json'),main:resolve(compiled,original.main),assets:{...original.assets,directory:resolve(compiled,original.assets.directory)},services:original.services.map(s=>({...s,service:'kortex-api-staging'})),vars:{WEB_ORIGIN:web,PUBLIC_RUN_ORIGIN:run},workers_dev:true,preview_urls:false};
writeFileSync(resolve(dir,'site.json'),JSON.stringify(siteConfig));
const runConfig={name:'kortex-run-staging',main:resolve(root,'runtime/web/worker.ts'),compatibility_date:'2026-10-26',send_metrics:false,workers_dev:true,preview_urls:false,assets:{directory:resolve(root,'target/web-dist'),binding:'ASSETS',run_worker_first:true},services:[{binding:'API_WORKER',service:'kortex-api-staging'}],vars:{PUBLIC_WEB_ORIGIN:web,RUN_ORIGIN:run,TRUST_KEYS:JSON.stringify([...existingKeys,releaseKey])}};
writeFileSync(resolve(dir,'runtime.json'),JSON.stringify(runConfig));
function buildHash(){const base=resolve(root,'website/dist'),files=[];function walk(dir){for(const item of readdirSync(dir,{withFileTypes:true})){const path=resolve(dir,item.name);if(item.isDirectory())walk(path);else files.push(path);}}walk(base);return hash(Buffer.concat(files.sort().flatMap(p=>[Buffer.from(relative(base,p)),readFileSync(p)])));}
async function deployment(){const data=await cf('GET',`/accounts/${account}/workers/scripts/kortex-web-staging/deployments`);return data.deployments[0].id;}
function jwt(){const n=Math.floor(Date.now()/1000),h=Buffer.from(JSON.stringify({alg:'EdDSA',kid,typ:'JWT'})).toString('base64url'),b=Buffer.from(JSON.stringify({iss:policy.issuer,aud:api,sub:policy.subject,repository:policy.repository,repository_id:policy.repository_id,ref:policy.ref,workflow_ref:policy.workflow_ref,iat:n,exp:n+300,jti:randomUUID()})).toString('base64url'),text=h+'.'+b;return text+'.'+sign(null,Buffer.from(text),operator.privateKey).toString('base64url');}
async function post(path,body){const r=await fetch(api+'/api/v1/'+path,{method:'POST',headers:{Authorization:'Bearer '+jwt(),'Content-Type':'application/json'},body:JSON.stringify(body),signal:AbortSignal.timeout(30000)});const data=await r.json();assert.equal(r.status,200,JSON.stringify(data));return data;}
async function get(url){return fetch(url,{headers:{'Cache-Control':'no-cache'},signal:AbortSignal.timeout(30000)});}
async function wait(check,seconds=65){const start=Date.now();while(Date.now()-start<seconds*1000){try{if(await check())return Date.now()-start;}catch{}await new Promise(done=>setTimeout(done,1000));}throw Error('Staging visibility timeout');}
async function sitemapHas(){const index=await get(web+'/sitemap-index.xml');assert.equal(index.status,200);for(const loc of (await index.text()).matchAll(/<loc>([^<]+)<\/loc>/g)){const r=await get(loc[1]);if((await r.text()).includes('/tools/'+tool+'</loc>'))return true;}return false;}
let platform,adminChanged=false,publishedNow=false,revoked=false;
const oldToken=process.env.CLOUDFLARE_API_TOKEN,oldAccount=process.env.CLOUDFLARE_ACCOUNT_ID;
try{
 cli(['d1','migrations','apply','DB','--remote','--config',apiPath]);cli(['deploy','--config',apiPath]);adminChanged=true;phase('staging_api_and_sitemap_binding_deployed');
 process.env.CLOUDFLARE_API_TOKEN=token;process.env.CLOUDFLARE_ACCOUNT_ID=account;
 platform=await getPlatformProxy({configPath:apiPath,remoteBindings:true,persist:false,envFiles:[]});
 await platform.env.DB.prepare('INSERT INTO signing_keys(id,public_key_base64,trust_version,not_before,expires) VALUES(?,?,1,?,?)').bind(keyId,releaseKey.public_key_base64,releaseKey.not_before,releaseKey.expires).run();
 const put=await platform.env.TOOLS_BUCKET.put(published.r2_key,bytes,{onlyIf:{etagDoesNotMatch:'*'},httpMetadata:{contentType:'application/vnd.kortex.tool+zip'}});assert.ok(put);
 cli(['deploy','--config',resolve(dir,'runtime.json')]);cli(['deploy','--config',resolve(dir,'site.json')]);
 await wait(async()=>(await get(web+'/')).status===200);phase('website_and_runtime_live',{web,run});
 report.site_build_before=buildHash();report.site_deployment_before=await deployment();assert.equal((await get(web+'/tools/'+tool)).status,404);assert.equal(await sitemapHas(),false);
 await post('admin/publications',published);publishedNow=true;const began=Date.now();
 const visibility=await wait(async()=>(await get(web+'/tools/'+tool)).status===200);assert.equal((await get(api+'/api/v1/tools/'+tool+'/actions')).status,200);await wait(sitemapHas);report.visibility_ms=Date.now()-began;assert.ok((await(await get(web+'/tools?q='+encodeURIComponent(manifest.name))).text()).includes('/tools/'+tool));
 phase('hot_publication_visible',{tool,visibility_ms:visibility});
 report.browsers=[];
 for(const [name,type] of [['chromium',chromium],['webkit',webkit]]){const browser=await type.launch();try{const page=await browser.newPage({viewport:{width:1353,height:882}});await page.goto(web+'/tools/'+tool);await page.getByRole('link',{name:'Ouvrir',exact:true}).click();await page.locator('[data-runtime-viewer][data-state=ready]').waitFor({timeout:45000});const frame=page.frameLocator('iframe');await frame.getByLabel('Premier nombre',{exact:true}).fill('25');await frame.getByLabel('Deuxième nombre',{exact:true}).fill('48');await frame.getByRole('button',{name:'Multiplier',exact:true}).click();await frame.locator('[data-node=result]').filter({hasText:'Résultat : 1200'}).waitFor();await frame.locator('[data-node=result]').scrollIntoViewIfNeeded();await page.screenshot({path:resolve(root,`docs/evidence/hot-publication-${name}.png`)});report.browsers.push({name,result:1200});}finally{await browser.close();}}
 phase('signed_runtime_execution_verified');
 await post('admin/artifacts/'+sha256+'/revoke',{request_id:randomUUID(),reason:'Validation 11.6 terminée ; retrait du fixture de staging.'});revoked=true;
 const latency=await wait(async()=>(await get(web+'/tools/'+tool)).status===404&&(await get(web+'/app/'+tool)).status===404);await wait(async()=>!await sitemapHas());
 assert.equal((await get(run+'/embed-config.js?tool='+tool+'&version=1.0.0')).status,404);assert.equal((await get(run+'/api/v1/downloads/'+tool+'/1.0.0')).status,410);
 report.revocation_ms=latency;report.site_build_after=buildHash();report.site_deployment_after=await deployment();assert.equal(report.site_build_after,report.site_build_before);assert.equal(report.site_deployment_after,report.site_deployment_before);
 Object.assign(report,{tool_id:tool,sha256,passed:true});phase('revocation_and_unchanged_website_verified');
}catch(error){report.error=String(error.message).slice(0,1500);phase('failed',{error:report.error});process.exitCode=1;}
finally{
 if(publishedNow&&!revoked)try{await post('admin/artifacts/'+sha256+'/revoke',{request_id:randomUUID(),reason:'Nettoyage du fixture après échec du test 11.6.'});}catch(error){report.cleanup_error=String(error.message);}
 if(platform)await platform.dispose();
 if(adminChanged){try{apiConfig.vars.ADMIN_AUTH_POLICY=base.vars.ADMIN_AUTH_POLICY;writeFileSync(apiPath,JSON.stringify(apiConfig));cli(['deploy','--config',apiPath]);report.operator_policy_restored=true;}catch(error){report.operator_policy_restore_error=String(error.message);report.passed=false;process.exitCode=1;}}
 if(oldToken===undefined)delete process.env.CLOUDFLARE_API_TOKEN;else process.env.CLOUDFLARE_API_TOKEN=oldToken;if(oldAccount===undefined)delete process.env.CLOUDFLARE_ACCOUNT_ID;else process.env.CLOUDFLARE_ACCOUNT_ID=oldAccount;
 writeFileSync(proof,JSON.stringify(report,null,2)+'\n');
}
