import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {spawnSync,spawn} from 'node:child_process';
import {randomUUID,randomBytes,generateKeyPairSync,createPrivateKey,createPublicKey,sign} from 'node:crypto';
import {createServer} from 'node:http';
import {getPlatformProxy} from 'wrangler';
import {prepareRelease} from './release-tool.mjs';
const phase135=process.env.KORTEX_PHASE135==='1',evidencePrefix=phase135?'desktop-store135':'desktop-lifecycle';
const root=resolve(import.meta.dirname,'..'),dir=resolve(root,'target/desktop-lifecycle','run-'+randomUUID().slice(0,8));mkdirSync(dir,{recursive:true});
const json=p=>JSON.parse(readFileSync(p,'utf8')),id='desktop-update-'+randomUUID().slice(0,8),proof={date:new Date().toISOString(),tool_id:id,staging_only:true,production_touched:false,mail_touched:false,published:[],uploaded:[],passed:false};
const proofFile=resolve(root,'docs/evidence/'+evidencePrefix+'-staging.json');const save=()=>writeFileSync(proofFile,JSON.stringify(proof,null,2)+'\n');
function cli(args){const r=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,...(args[0]==='whoami'?[]:['--profile','kortex'])],{cwd:root,encoding:'utf8',windowsHide:true,timeout:180000,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'}});if(r.status!==0)throw Error('Cloudflare '+args[0]+' failed: '+r.stderr.slice(-800));return r.stdout}
const owner=json(resolve(root,'target/cloud-staging-resources.json')),auth=JSON.parse(cli(['whoami','--json']));assert.equal(auth.accounts[0].id,owner.account);const token=JSON.parse(cli(['auth','token','--json'])).token;
const api='https://kortex-api-staging.userpgl.workers.dev';
const sourceDir=resolve(dir,'source');mkdirSync(resolve(sourceDir,'src'),{recursive:true});mkdirSync(resolve(sourceDir,'assets'),{recursive:true});
writeFileSync(resolve(sourceDir,'Cargo.toml'),'[package]\nname="kortex-desktop-lifecycle-fixture"\nversion="1.0.0"\nedition="2024"\nrust-version="1.99"\npublish=false\n[workspace]\n[lib]\ncrate-type=["cdylib","rlib"]\n[dependencies]\nkortex-sdk={path="../../../crates/kortex-sdk"}\nserde_json="=1.0.154"\n'.replace('../../../crates/kortex-sdk',resolve(root,'crates/kortex-sdk').split(String.fromCharCode(92)).join('/')));
writeFileSync(resolve(sourceDir,'src/lib.rs'),readFileSync(resolve(root,'tools/convertisseur-casse/src/lib.rs')));
let locked=spawnSync('cargo',['generate-lockfile','--manifest-path',resolve(sourceDir,'Cargo.toml'),'--offline'],{cwd:root,encoding:'utf8',windowsHide:true});assert.equal(locked.status,0,locked.stderr);
const seed=randomBytes(32),privateKey=createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'}),offlineRoot=generateKeyPairSync('ed25519'),now=Date.now(),keyId=id+'-release';
const anchor={id:id+'-root',public_key_base64:offlineRoot.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64')},key={id:keyId,public_key_base64:createPublicKey(privateKey).export({format:'der',type:'spki'}).subarray(-32).toString('base64'),not_before:now-1000,expires:now+86400000,revoked:false};
const keys={schema_version:1,version:1,issued_at:now-1000,expires:key.expires,root_key_id:anchor.id,keys:[key],signature:''};keys.signature=sign(null,Buffer.from('kortex-keys-v1\n1\n'+keys.issued_at+'\n'+keys.expires+'\n'+anchor.id+'\n'+key.id+'|'+key.public_key_base64+'|'+key.not_before+'|'+key.expires+'|false\n'),offlineRoot.privateKey).toString('base64');
const rootPath=resolve(dir,'root.json'),keyPath=resolve(dir,'keys.json');writeFileSync(rootPath,JSON.stringify(anchor));writeFileSync(keyPath,JSON.stringify(keys));
const releases=[],oldSigning={CI:process.env.CI,KORTEX_SIGNING_KEY_ID:process.env.KORTEX_SIGNING_KEY_ID,KORTEX_SIGNING_KEY_SEED_BASE64:process.env.KORTEX_SIGNING_KEY_SEED_BASE64};
try{
 for(const version of ['1.0.0','1.1.0','1.2.0']){
 const manifest=json(resolve(root,'tools/convertisseur-casse/manifest.json')),contract=json(resolve(root,'tools/convertisseur-casse/contract.json'));
 Object.assign(manifest,{id,version,name:'Vérification Desktop',permissions:version==='1.1.0'?['file.read']:[],description:'Vérification temporaire des installations, mises à jour et retours arrière.'});contract.tool_id=id;
 writeFileSync(resolve(sourceDir,'manifest.json'),JSON.stringify(manifest));writeFileSync(resolve(sourceDir,'contract.json'),JSON.stringify(contract));writeFileSync(resolve(sourceDir,'assets/icon.svg'),readFileSync(resolve(root,'tools/convertisseur-casse/assets/icon.svg')));
 let guest=readFileSync(resolve(root,'tools/convertisseur-casse/src/lib.rs'),'utf8');
 if(version==='1.2.0'){const before=guest;guest=guest.replace('RequestKind::Init=>{*self=Self{running:true,mode:"lower".into(),..Self::default()};Ok(json!({}))}','RequestKind::Init=>Err("INITIALIZATION_FAILED")');assert.notEqual(guest,before);}
 writeFileSync(resolve(sourceDir,'src/lib.rs'),guest);process.env.CI='true';process.env.KORTEX_SIGNING_KEY_ID=keyId;process.env.KORTEX_SIGNING_KEY_SEED_BASE64=seed.toString('base64');
 const release=prepareRelease({tool:id,sourceDir,out:resolve(dir,'releases',version),root:rootPath,trust:keyPath,dryRun:false});releases.push(release);console.log('Prepared signed fixture '+version);
 }
}finally{seed.fill(0);for(const[k,v]of Object.entries(oldSigning))if(v===undefined)delete process.env[k];else process.env[k]=v;}
writeFileSync(resolve(root,'docs/evidence/'+evidencePrefix+'-artifacts.json'),JSON.stringify(releases.map(r=>({version:r.version,sha256:r.sha256,descriptor:json(r.descriptor),host_trust:json(r.host_trust)})),null,2)+'\n');
const operator=generateKeyPairSync('ed25519'),kid=id+'-operator',policy={issuer:'urn:kortex:staging-operator',audience:api,subject:'kortex-staging-operator',repository:'local/kortex-staging',repository_id:'0',ref:'refs/heads/staging',workflow_ref:'local/kortex-staging/staging-publication@refs/heads/staging',actions:['publish','revoke','rollback'],jwks:{keys:[{...operator.publicKey.export({format:'jwk'}),kid,alg:'EdDSA',use:'sig'}]}};
const src=json(resolve(root,'api/wrangler.jsonc')),base=src.env.staging;assert.equal(base.name,owner.worker);assert.equal(base.d1_databases[0].database_id,owner.database_id);assert.equal(base.r2_buckets[0].bucket_name,owner.bucket);
const config={...base,main:resolve(root,'api/src/index.ts'),compatibility_date:src.compatibility_date,vars:{...base.vars,ADMIN_AUTH_POLICY:JSON.stringify(policy)},d1_databases:base.d1_databases.map(d=>({...d,migrations_dir:resolve(root,'database/migrations')}))};const configPath=resolve(dir,'api.json');writeFileSync(configPath,JSON.stringify(config));
function jwt(){const now=Math.floor(Date.now()/1000),h=Buffer.from(JSON.stringify({alg:'EdDSA',kid,typ:'JWT'})).toString('base64url'),b=Buffer.from(JSON.stringify({iss:policy.issuer,aud:api,sub:policy.subject,repository:policy.repository,repository_id:policy.repository_id,ref:policy.ref,workflow_ref:policy.workflow_ref,iat:now,exp:now+300,jti:randomUUID()})).toString('base64url'),text=h+'.'+b;return text+'.'+sign(null,Buffer.from(text),operator.privateKey).toString('base64url')}
async function post(path,body){const r=await fetch(api+'/api/v1/'+path,{method:'POST',headers:{Authorization:'Bearer '+jwt(),'Content-Type':'application/json'},body:JSON.stringify(body),signal:AbortSignal.timeout(30000)});const data=await r.json();assert.equal(r.status,200,JSON.stringify(data));return data}
let platform,changed=false,control;const oldToken=process.env.CLOUDFLARE_API_TOKEN,oldAccount=process.env.CLOUDFLARE_ACCOUNT_ID;
async function publish(version){const r=releases.find(r=>r.version===version);const request=json(r.publication_request);if(proof.published.includes(version))return;const put=await platform.env.TOOLS_BUCKET.put(request.r2_key,readFileSync(r.archive),{onlyIf:{etagDoesNotMatch:'*'}});assert.ok(put);proof.uploaded.push(version);await post('admin/publications',request);proof.published.push(version);save();console.log('Published fixture '+version);}
try{
 changed=true;cli(['deploy','--config',configPath]);process.env.CLOUDFLARE_API_TOKEN=token;process.env.CLOUDFLARE_ACCOUNT_ID=owner.account;
 platform=await getPlatformProxy({configPath,remoteBindings:true,persist:false,envFiles:[]});
 await platform.env.DB.prepare('INSERT INTO signing_keys(id,public_key_base64,trust_version,not_before,expires) VALUES(?,?,1,?,?)').bind(key.id,key.public_key_base64,key.not_before,key.expires).run();
 if(!phase135)await publish('1.0.0');
 const nonce=randomUUID();control=createServer(async(req,res)=>{if(req.headers['x-kortex-test']!==nonce){res.writeHead(403);res.end();return}try{if(req.url==='/first')await publish('1.0.0');else if(req.url==='/next')await publish('1.1.0');else if(req.url==='/broken')await publish('1.2.0');else if(req.url==='/restore-good')await post('admin/tools/'+id+'/rollback',{request_id:randomUUID(),version:'1.1.0',expected_current_version:'1.2.0'});else throw Error('Unknown test operation');res.writeHead(200,{'Content-Type':'application/json'});res.end('{"ok":true}')}catch(e){res.writeHead(500);res.end(String(e.message))}});
 await new Promise(done=>control.listen(0,'127.0.0.1',done));const controlOrigin='http://127.0.0.1:'+control.address().port;
 const combinedTrust=resolve(dir,'combined-trust.json');writeFileSync(combinedTrust,JSON.stringify([...json(resolve(root,'desktop/assets/staging-trust.json')),json(releases[0].host_trust)]));
 const info={phase135,evidence_prefix:evidencePrefix,tool_id:id,name:'Vérification Desktop',data_dir:resolve(dir,'app-'+id),trust_path:phase135?combinedTrust:releases[0].host_trust,control_origin:controlOrigin};const infoPath=resolve(dir,'run.json');writeFileSync(infoPath,JSON.stringify(info));
 const beforeBinary=readFileSync(resolve(root,'target/debug/kortex-desktop.exe')).toString('base64');
 const result=await new Promise(done=>{const child=spawn('pwsh',['-NoProfile','-File','tests/desktop/lifecycle.ps1'],{cwd:root,stdio:'inherit',windowsHide:true,env:{...process.env,KORTEX_LIFECYCLE_INFO:infoPath,KORTEX_LIFECYCLE_NONCE:nonce}});child.on('exit',code=>done(code));child.on('error',()=>done(1));});assert.equal(result,0,'Windows lifecycle gates failed');proof.passed=true;proof.desktop_binary_unchanged=beforeBinary===readFileSync(resolve(root,'target/debug/kortex-desktop.exe')).toString('base64');assert.ok(proof.desktop_binary_unchanged);
}catch(e){proof.error=String(e.message).slice(0,1200);console.error(proof.error);process.exitCode=1;}
finally{
 if(control){control.closeAllConnections();await new Promise(done=>control.close(done));}
 if(platform)try{for(const version of [...proof.published].reverse()){const r=releases.find(r=>r.version===version);await post('admin/artifacts/'+r.sha256+'/revoke',{request_id:randomUUID(),reason:'Fin du test temporaire Desktop 13.2–13.4.'});await platform.env.TOOLS_BUCKET.delete(json(r.publication_request).r2_key);assert.equal(await platform.env.TOOLS_BUCKET.head(json(r.publication_request).r2_key),null);}
 for(const version of proof.uploaded){const r=releases.find(r=>r.version===version);await platform.env.TOOLS_BUCKET.delete(json(r.publication_request).r2_key);assert.equal(await platform.env.TOOLS_BUCKET.head(json(r.publication_request).r2_key),null);}
 await platform.env.DB.prepare('INSERT INTO revocations(request_id,signing_key_id,reason,created_at) VALUES(?,?,?,?)').bind(randomUUID(),key.id,'Fin du test temporaire Desktop 13.2–13.4.',Date.now()).run();assert.equal(await platform.env.DB.prepare('SELECT revoked FROM signing_keys WHERE id=?').bind(key.id).first('revoked'),1);proof.fixtures_revoked=true;proof.objects_removed=true;proof.temporary_key_revoked=true;assert.equal((await fetch(api+'/api/v1/tools/'+id)).status,404);
 }catch(e){proof.cleanup_error=String(e.message);proof.passed=false;process.exitCode=1;}
 if(platform)await platform.dispose();
 if(changed)try{config.vars.ADMIN_AUTH_POLICY=base.vars.ADMIN_AUTH_POLICY;writeFileSync(configPath,JSON.stringify(config));cli(['deploy','--config',configPath]);proof.operator_policy_restored=true;}catch(e){proof.restore_error=String(e.message);proof.passed=false;process.exitCode=1;}
 if(oldToken===undefined)delete process.env.CLOUDFLARE_API_TOKEN;else process.env.CLOUDFLARE_API_TOKEN=oldToken;if(oldAccount===undefined)delete process.env.CLOUDFLARE_ACCOUNT_ID;else process.env.CLOUDFLARE_ACCOUNT_ID=oldAccount;save();
}
