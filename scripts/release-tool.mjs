// Local preparation only. No upload, network client or Cloudflare command exists here.
import {spawnSync} from 'node:child_process';
import {readFileSync,writeFileSync,mkdirSync,existsSync,realpathSync,lstatSync} from 'node:fs';
import {resolve,sep,dirname} from 'node:path';
import {randomBytes,createPrivateKey,createPublicKey,generateKeyPairSync,sign,randomUUID,createHash} from 'node:crypto';
import {fileURLToPath,pathToFileURL} from 'node:url';
import Ajv2020 from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'..');
const json=p=>JSON.parse(readFileSync(p,'utf8'));
const ajv=new Ajv2020({strict:false});addFormats(ajv);
const schemas=Object.fromEntries(['manifest','contract','artifact'].map(name=>[name,ajv.compile(json(resolve(root,`shared/schemas/${name}.schema.json`)))]));
export function prepareRelease(options){
 const tool=options.tool;if(!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(tool??''))throw Error('INVALID_TOOL');
 const out=resolve(options.out??resolve(root,'target/staging',tool));const target=resolve(root,'target');if(!out.startsWith(target+sep))throw Error('STAGING_MUST_STAY_UNDER_TARGET');
 mkdirSync(out,{recursive:true});if(!realpathSync(out).startsWith(realpathSync(target)+sep))throw Error('STAGING_LINK_REFUSED');
 const previous=existsSync(resolve(out,'publication-request.json'))?json(resolve(out,'publication-request.json')):null;
 if(previous&&(!existsSync(resolve(out,'release-report.json'))||json(resolve(out,'release-report.json')).dry_run!==!!options.dryRun))throw Error('STAGING_PROFILE_CONFLICT');
 if(options.requestId&&!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(options.requestId))throw Error('INVALID_REQUEST_ID');
 if(previous&&options.requestId&&options.requestId!==previous.request_id)throw Error('STAGING_REQUEST_ID_CONFLICT');
 const env={...process.env,CARGO_NET_OFFLINE:'true'};for(const name of Object.keys(env))if(/TOKEN|SECRET|PASSWORD|PEPPER|PRIVATE_KEY|SIGNING_KEY_SEED/.test(name))delete env[name];
 const calls=[];const cli=(args,extra={})=>{const signer=args[0]==='sign',binary=signer?resolve(root,env.CARGO_TARGET_DIR||'target','debug','kortex-xtask'+(process.platform==='win32'?'.exe':'')):'cargo';if(signer&&!existsSync(binary))throw Error('PREBUILT_SIGNER_REQUIRED');const r=spawnSync(binary,[...(signer?[]:['run','-p','kortex-xtask','--locked','--']),...args,...(options.sourceDir&&['build','validate','package'].includes(args[0])?['--source-dir',resolve(options.sourceDir)]:[]),'--json'],{cwd:root,encoding:'utf8',env:{...env,...extra}});if(r.status!==0)throw Error(`CLI_${args[0]}_FAILED`);calls.push(args[0]);return JSON.parse(r.stdout)};
 cli(['build','--tool',tool]);cli(['validate','--tool',tool]);const packed=cli(['package','--tool',tool,'--out',resolve(out,'packages')]).results[0];
 const {manifest,contract,sha256}=packed;
 if(createHash('sha256').update(readFileSync(packed.archive)).digest('hex')!==sha256)throw Error('HASH_MISMATCH');calls.push('hash');if(manifest.id!==tool||contract.tool_id!==tool)throw Error('IDENTITY_MISMATCH');
 if(previous&&(previous.descriptor.sha256!==sha256||previous.manifest.version!==manifest.version||previous.manifest.id!==manifest.id))throw Error('IMMUTABLE_VERSION_CONFLICT');
 const proofPath=packed.archive+'.json',rootPath=resolve(out,'root.json'),keysPath=resolve(out,'keys.json');
 if(!previous){
  if(options.dryRun){
   const seed=randomBytes(32),privateKey=createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'});const offline=generateKeyPairSync('ed25519');const pub=k=>createPublicKey(k).export({format:'der',type:'spki'}).subarray(-32).toString('base64');
   const anchor={id:'dry-run-root',public_key_base64:pub(offline.privateKey)},now=Date.now();const keys={schema_version:1,version:1,issued_at:now-1000,expires:now+86400000,root_key_id:anchor.id,keys:[{id:'dry-run-release',public_key_base64:pub(privateKey),not_before:now-1000,expires:now+86400000,revoked:false}],signature:''};
   const k=keys.keys[0];const text=`kortex-keys-v1\n1\n${keys.issued_at}\n${keys.expires}\n${anchor.id}\n${k.id}|${k.public_key_base64}|${k.not_before}|${k.expires}|false\n`;keys.signature=sign(null,Buffer.from(text),offline.privateKey).toString('base64');
   writeFileSync(rootPath,JSON.stringify(anchor,null,2));writeFileSync(keysPath,JSON.stringify(keys,null,2));
   try{cli(['sign','--artifact',packed.archive],{CI:'true',KORTEX_SIGNING_KEY_ID:k.id,KORTEX_SIGNING_KEY_SEED_BASE64:seed.toString('base64')});}finally{seed.fill(0)}
  }else{
   if(process.env.CI!=='true'||!process.env.KORTEX_SIGNING_KEY_SEED_BASE64||!options.root||!options.trust)throw Error('CI_SECRET_AND_EXPLICIT_TRUST_REQUIRED');
   writeFileSync(rootPath,readFileSync(options.root));writeFileSync(keysPath,readFileSync(options.trust));cli(['sign','--artifact',packed.archive],{CI:'true',KORTEX_SIGNING_KEY_SEED_BASE64:process.env.KORTEX_SIGNING_KEY_SEED_BASE64});
  }
 }
 cli(['verify','--artifact',proofPath,'--root',rootPath,'--trust',keysPath,'--min-trust-version','1']);
 const descriptor=json(proofPath),request={descriptor,manifest,contract,r2_key:`tools/${tool}/${manifest.version}/${sha256}.tool`,request_id:previous?.request_id??options.requestId??randomUUID()};
 validatePublication(request);const serialized=JSON.stringify(request,null,2)+'\n';const requestPath=resolve(out,'publication-request.json');if(previous&&JSON.stringify(previous)!==JSON.stringify(request))throw Error('STAGING_CONTENT_CONFLICT');if(!previous)writeFileSync(requestPath,serialized,{flag:'wx'});
 // This bundle is host configuration, supplied by the test operator, never by the guest.
 const bundle={root:json(rootPath),document:json(keysPath),minimum_version:1};writeFileSync(resolve(out,'host-trust.json'),JSON.stringify(bundle,null,2));
 const report={prepared:true,dry_run:!!options.dryRun,remote_writes:0,cargo_offline:true,tool_id:tool,version:manifest.version,sha256,archive:packed.archive,descriptor:proofPath,publication_request:requestPath,host_trust:resolve(out,'host-trust.json'),calls,idempotent:!!previous};
 writeFileSync(resolve(out,'release-report.json'),JSON.stringify(report,null,2)+'\n');return report;
}
export function validatePublication(r){
 if(!r||Object.keys(r).sort().join()!==['descriptor','manifest','contract','r2_key','request_id'].sort().join())throw Error('INVALID_PUBLICATION');
 if(!schemas.manifest(r.manifest)||!schemas.contract(r.contract)||!schemas.artifact(r.descriptor))throw Error('INVALID_PUBLICATION_SCHEMA');
 const d=r.descriptor,m=r.manifest;if(!d||!m||r.contract?.tool_id!==m.id||d.tool_id!==m.id||d.version!==m.version||d.abi_version!==m.abi_version||!/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i.test(r.request_id)||r.r2_key!==`tools/${m.id}/${m.version}/${d.sha256}.tool`||!/^[a-f0-9]{64}$/.test(d.sha256))throw Error('PUBLICATION_MISMATCH');
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){try{const args=process.argv.slice(2),o={};for(let i=0;i<args.length;i++){const k=args[i];if(k==='--dry-run'){o.dryRun=true;continue;}if(!['--tool','--out','--root','--trust','--request-id'].includes(k)||!args[i+1])throw Error('INVALID_ARGUMENT');o[{'--tool':'tool','--out':'out','--root':'root','--trust':'trust','--request-id':'requestId'}[k]]=args[++i];}console.log(JSON.stringify(prepareRelease(o)))}catch{console.error('Local release preparation failed; no remote publication performed.');process.exitCode=1}}
