import assert from 'node:assert/strict';
import {spawnSync} from 'node:child_process';
import {readFileSync,writeFileSync,mkdirSync,existsSync,appendFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash,randomBytes,randomUUID,generateKeyPairSync,createPrivateKey,createPublicKey,sign} from 'node:crypto';
import {getPlatformProxy} from 'wrangler';
import {prepareRelease} from './release-tool.mjs';
import {canonical,trustedFiles,storedZip} from './staging-fixtures.mjs';
const root=resolve(import.meta.dirname,'..'),out=resolve(root,'target/staging-cloud');mkdirSync(out,{recursive:true});
const json=path=>JSON.parse(readFileSync(path,'utf8')),hash=b=>createHash('sha256').update(b).digest('hex');
const ownershipPath=resolve(root,'target/cloud-staging-resources.json');let ownership=existsSync(ownershipPath)?json(ownershipPath):{};
function cli(args,extra={}){const r=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,...(args[0]==='whoami'?[]:['--profile','kortex'])],{cwd:root,encoding:'utf8',windowsHide:true,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false',...extra},timeout:180000});if(!args.includes('token'))appendFileSync(resolve(out,'wrangler.log'),r.stdout+'\n'+r.stderr+'\n');if(r.status!==0)throw Error('WRANGLER_'+args.slice(0,3).join('_')+'_FAILED: '+r.stderr.slice(-800));return r.stdout}
const identity=JSON.parse(cli(['whoami','--json']));assert.equal(identity.accounts?.length,1);const account=identity.accounts[0].id;
if(ownership.account&&ownership.account!==account)throw Error('STAGING_ACCOUNT_CHANGED');ownership.account=account;
const oauth=JSON.parse(cli(['auth','token','--json'])).token;
async function cf(method,path,body){const response=await fetch('https://api.cloudflare.com/client/v4'+path,{method,headers:{Authorization:'Bearer '+oauth,'Content-Type':'application/json'},body:body===undefined?undefined:JSON.stringify(body),signal:AbortSignal.timeout(30000)});const data=await response.json();if(!response.ok||data.success===false)throw Error('CLOUDFLARE_'+response.status+' '+JSON.stringify(data.errors??[]).slice(0,600));return data.result}
function saveOwnership(){writeFileSync(ownershipPath,JSON.stringify(ownership,null,2)+'\n')}
const report={date:new Date().toISOString(),staging_only:true,production_touched:false,mail_touched:false,phases:[],auth:'temporary pinned staging operator; not a GitHub OIDC run'};
function phase(name,data={}){report.phases.push({name,...data});writeFileSync(resolve(out,'report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({phase:name,...data}))}
let platform;const previousToken=process.env.CLOUDFLARE_API_TOKEN,previousAccount=process.env.CLOUDFLARE_ACCOUNT_ID;
try{
 const databases=await cf('GET',`/accounts/${account}/d1/database?per_page=100`),dbName='kortex-catalog-staging',bucketName='kortex-tools-staging',worker='kortex-api-staging';
 let database=databases.find(d=>d.name===dbName);if(database&&database.uuid!==ownership.database_id)throw Error('UNOWNED_STAGING_DATABASE');
 if(!database){database=await cf('POST',`/accounts/${account}/d1/database`,{name:dbName,primary_location_hint:'weur'});ownership.database_id=database.uuid;saveOwnership()}
 const buckets=await cf('GET',`/accounts/${account}/r2/buckets`);let bucket=buckets.buckets.find(b=>b.name===bucketName);
 if(bucket&&ownership.bucket!==bucketName)throw Error('UNOWNED_STAGING_BUCKET');
 if(!bucket){await cf('POST',`/accounts/${account}/r2/buckets`,{name:bucketName,locationHint:'weur'});ownership.bucket=bucketName;saveOwnership()}
 const managed=await cf('GET',`/accounts/${account}/r2/buckets/${bucketName}/domains/managed`),custom=await cf('GET',`/accounts/${account}/r2/buckets/${bucketName}/domains/custom`);
 assert.equal(managed.enabled,false);assert.equal(custom.domains?.length??0,0);phase('private_staging_resources_created');
 const scripts=await cf('GET',`/accounts/${account}/workers/scripts`);if(scripts.some(s=>s.id===worker)&&ownership.worker!==worker)throw Error('UNOWNED_STAGING_WORKER');
 const subdomain=await cf('GET',`/accounts/${account}/workers/subdomain`);if(!subdomain.subdomain)throw Error('WORKERS_DEV_SUBDOMAIN_NOT_CONFIGURED');const origin=`https://${worker}.${subdomain.subdomain}.workers.dev`;
 const operator=generateKeyPairSync('ed25519'),kid='staging-operator-'+randomUUID().slice(0,8),operatorPublic={...operator.publicKey.export({format:'jwk'}),kid,alg:'EdDSA',use:'sig'};
 const policy={issuer:'urn:kortex:staging-operator',audience:origin,subject:'kortex-staging-operator',repository:'local/kortex-staging',repository_id:'0',ref:'refs/heads/staging',workflow_ref:'local/kortex-staging/staging-publication@refs/heads/staging',actions:['publish','revoke','rollback'],jwks:{keys:[operatorPublic]}};
 const configPath=resolve(root,'api/wrangler.jsonc'),config=json(configPath);config.env.staging={name:worker,workers_dev:true,preview_urls:false,vars:{KORTEX_ENV:'staging',PUBLIC_WEB_ORIGIN:'https://staging.kortex.site',ADMIN_AUTH_POLICY:JSON.stringify(policy)},d1_databases:[{binding:'DB',database_name:dbName,database_id:database.uuid,migrations_dir:'../database/migrations',remote:true}],r2_buckets:[{binding:'TOOLS_BUCKET',bucket_name:bucketName,remote:true}]};
 writeFileSync(configPath,JSON.stringify(config,null,2)+'\n');process.env.CLOUDFLARE_ACCOUNT_ID=account;
 cli(['d1','migrations','apply','DB','--remote','--env','staging','--config',configPath]);phase('three_remote_migrations_applied');
 cli(['deploy','--env','staging','--config',configPath]);ownership.worker=worker;saveOwnership();
 await cf('PUT',`/accounts/${account}/workers/scripts/${worker}/secrets`,{name:'CURSOR_SIGNING_KEY',text:randomBytes(32).toString('hex'),type:'secret_text'});
 report.api_origin=origin;report.resources={worker,database_name:dbName,database_id:database.uuid,bucket:bucketName};phase('staging_api_deployed',{origin});
 const pipeline=prepareRelease({tool:'calculatrice',dryRun:true,out:resolve(root,'target/staging/cloud-pipeline')});phase('canonical_pipeline_artifact_prepared',{sha256:pipeline.sha256});
 process.env.CLOUDFLARE_API_TOKEN=oauth;
 platform=await getPlatformProxy({configPath,environment:'staging',remoteBindings:true,persist:false,envFiles:[]});
 const db=platform.env.DB,r2=platform.env.TOOLS_BUCKET,original=json(pipeline.publication_request),originalTrust=json(pipeline.host_trust),originalKey=originalTrust.document.keys[0];
 const seed=randomBytes(32),releasePrivate=createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'}),releaseId='staging-release-'+randomUUID().slice(0,8),now=Date.now();
 const releasePublic=createPublicKey(releasePrivate).export({format:'der',type:'spki'}).subarray(-32).toString('base64'),releaseKey={id:releaseId,public_key_base64:releasePublic,not_before:now-60000,expires:now+86400000,revoked:false};
 for(const k of [originalKey,releaseKey]){await db.prepare('INSERT INTO signing_keys(id,public_key_base64,trust_version,not_before,expires) SELECT ?,?,1,?,? WHERE NOT EXISTS(SELECT 1 FROM signing_keys WHERE id=?)').bind(k.id,k.public_key_base64,k.not_before,k.expires,k.id).run();assert.equal(await db.prepare('SELECT public_key_base64 FROM signing_keys WHERE id=?').bind(k.id).first('public_key_base64'),k.public_key_base64)}
 async function upload(key,bytes){const put=await r2.put(key,bytes,{onlyIf:{etagDoesNotMatch:'*'},httpMetadata:{contentType:'application/vnd.kortex.tool+zip'}});if(!put){const existing=await r2.get(key);assert.ok(existing);assert.equal(hash(Buffer.from(await existing.arrayBuffer())),hash(bytes))}assert.equal(await r2.put(key,new Uint8Array([0]),{onlyIf:{etagDoesNotMatch:'*'}}),null)}
 function jwt(){const now=Math.floor(Date.now()/1000),h=Buffer.from(JSON.stringify({alg:'EdDSA',kid,typ:'JWT'})).toString('base64url'),b=Buffer.from(JSON.stringify({iss:policy.issuer,aud:policy.audience,sub:policy.subject,repository:policy.repository,repository_id:policy.repository_id,ref:policy.ref,workflow_ref:policy.workflow_ref,iat:now,exp:now+300,jti:randomUUID()})).toString('base64url'),text=h+'.'+b;return text+'.'+sign(null,Buffer.from(text),operator.privateKey).toString('base64url')}
 async function request(path,body){return fetch(origin+'/api/v1/'+path,{method:body?'POST':'GET',headers:body?{Authorization:'Bearer '+jwt(),'Content-Type':'application/json'}:{},body:body?JSON.stringify(body):undefined,signal:AbortSignal.timeout(30000)})}
 async function post(path,body){const r=await request(path,body),payload=await r.json();if(r.status!==200)throw Error('STAGING_ADMIN_'+r.status+' '+JSON.stringify(payload));return payload}
 let detailResponse;for(let i=0;i<15;i++){try{detailResponse=await request('health');if(detailResponse.status===200)break}catch{}await new Promise(done=>setTimeout(done,1000))}assert.equal(detailResponse?.status,200);
 const unauthorized=await fetch(origin+'/api/v1/admin/publications',{method:'POST',headers:{'Content-Type':'application/json'},body:'{}'});assert.equal(unauthorized.status,401);
 const bytes=readFileSync(pipeline.archive);await upload(original.r2_key,bytes);
 const existing=await request('tools/calculatrice');if(existing.status===404)await post('admin/publications',original);else assert.equal(existing.status,200);
 const current=await (await request('tools/calculatrice')).json();assert.equal(current.tool.version,'1.0.0');assert.equal(current.tool.sha256,pipeline.sha256);phase('first_publication_verified',{tool_id:'calculatrice',version:'1.0.0',sha256:pipeline.sha256});
 const versions=await db.prepare("SELECT version FROM tool_versions WHERE tool_id='calculatrice'").all();let index=0;while(versions.results.some(v=>v.version===`1.1.${index}`))index++;const secondVersion=`1.1.${index}`;
 const secondManifest={...original.manifest,version:secondVersion},files=trustedFiles(bytes);files.set('manifest.json',Buffer.from(canonical(secondManifest)));const secondBytes=storedZip(files),secondPath=resolve(out,`calculatrice-${secondVersion}.tool`);writeFileSync(secondPath,secondBytes);
 // The same Rust admission/signing CLI verifies the staging-only new container before signing.
 const secondProof=resolve(out,`calculatrice-${secondVersion}-${releaseId}.json`);
 const signed=spawnSync('cargo',['run','-p','kortex-xtask','--locked','--','sign','--artifact',secondPath,'--out',secondProof,'--json'],{cwd:root,encoding:'utf8',windowsHide:true,env:{...process.env,CI:'true',CARGO_NET_OFFLINE:'true',KORTEX_SIGNING_KEY_ID:releaseId,KORTEX_SIGNING_KEY_SEED_BASE64:seed.toString('base64')},timeout:180000});seed.fill(0);if(signed.status!==0)throw Error('STAGING_SECOND_SIGNING_FAILED');
 const secondDescriptor=json(secondProof),second={descriptor:secondDescriptor,manifest:secondManifest,contract:original.contract,r2_key:`tools/calculatrice/${secondVersion}/${hash(secondBytes)}.tool`,request_id:randomUUID()};await upload(second.r2_key,secondBytes);await post('admin/publications',second);
 assert.equal((await (await request('tools/calculatrice')).json()).tool.version,secondVersion);
 await post('admin/tools/calculatrice/rollback',{request_id:randomUUID(),version:'1.0.0',expected_current_version:secondVersion});assert.equal((await (await request('tools/calculatrice')).json()).tool.version,'1.0.0');
 await post('admin/artifacts/'+secondDescriptor.sha256+'/revoke',{request_id:randomUUID(),reason:'Staging roundtrip revocation test; healthy 1.0.0 retained'});
 assert.equal((await request('downloads/calculatrice/'+secondVersion)).status,410);assert.equal((await request('tools/calculatrice')).status,200);phase('rollback_and_revocation_verified',{revoked_version:secondVersion});
 const download=await request('downloads/calculatrice/1.0.0');assert.equal(download.status,200);const downloaded=Buffer.from(await download.arrayBuffer());assert.equal(hash(downloaded),current.descriptor.sha256);assert.deepEqual(downloaded,bytes);
 const archive=resolve(out,'downloaded-calculatrice.tool');writeFileSync(archive,downloaded);writeFileSync(archive+'.json',JSON.stringify(current.descriptor,null,2)+'\n');
 const trustKeys=[originalKey,releaseKey],rootKey=generateKeyPairSync('ed25519'),anchor={id:'staging-gate-root',public_key_base64:rootKey.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64')},document={schema_version:1,version:1,issued_at:Date.now()-1000,expires:Math.min(...trustKeys.map(k=>k.expires)),root_key_id:anchor.id,keys:trustKeys,signature:''};
 const keyLines=[...trustKeys].sort((a,b)=>a.id<b.id?-1:1).map(k=>`${k.id}|${k.public_key_base64}|${k.not_before}|${k.expires}|false\n`).join('');document.signature=sign(null,Buffer.from(`kortex-keys-v1\n1\n${document.issued_at}\n${document.expires}\n${anchor.id}\n${keyLines}`),rootKey.privateKey).toString('base64');
 const hostTrust=resolve(out,'host-trust.json');writeFileSync(hostTrust,JSON.stringify({root:anchor,document,minimum_version:1},null,2)+'\n');
 Object.assign(report,{archive,descriptor:archive+'.json',host_trust:hostTrust,sha256:hash(downloaded),tool_id:'calculatrice',version:'1.0.0',download_mime:download.headers.get('content-type'),revoked_version:secondVersion,cloud_roundtrip_passed:true,hosts_verified:false});phase('cloud_roundtrip_passed',{sha256:report.sha256});
}catch(error){report.error=String(error.message).slice(0,1000);report.cloud_roundtrip_passed=false;writeFileSync(resolve(out,'report.json'),JSON.stringify(report,null,2)+'\n');console.error(report.error);process.exitCode=1}
finally{if(platform)await platform.dispose();if(previousToken===undefined)delete process.env.CLOUDFLARE_API_TOKEN;else process.env.CLOUDFLARE_API_TOKEN=previousToken;if(previousAccount===undefined)delete process.env.CLOUDFLARE_ACCOUNT_ID;else process.env.CLOUDFLARE_ACCOUNT_ID=previousAccount}
