import {waitForApi} from './api-readiness.mjs';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash,createPrivateKey,sign} from 'node:crypto';
import {resolve} from 'node:path';
import {context,protection} from './ci-publish-tool.mjs';
import {requireQuality} from './ci-quality.mjs';
import {publishRuntime} from './publish-runtime.mjs';
import {canonicalRuntime} from '../shared/typescript/src/runtime.ts';
const env=process.env;const deployment=context(env,'staging');await requireQuality(env);await waitForApi(env,deployment);await protection(env,'staging');
const folder='target/desktop-package',descriptor=JSON.parse(readFileSync(folder+'/desktop-release.json')),smoke=JSON.parse(readFileSync(folder+'/installer-smoke.json'));
if(!smoke.passed||!smoke.installed||!smoke.uninstalled||descriptor.channel!=='staging'||descriptor.platform!=='windows-x64'||descriptor.code_signed!==false||!/^[A-Za-z0-9._-]+\.exe$/.test(descriptor.filename)||!/^\d+\.\d+\.\d+$/.test(descriptor.version))throw Error('DESKTOP_ARTIFACT_NOT_VALIDATED');
const bytes=readFileSync(resolve(folder,descriptor.filename));if(createHash('sha256').update(bytes).digest('hex')!==descriptor.sha256||bytes.length!==descriptor.bytes)throw Error('DESKTOP_ARTIFACT_HASH');
const repository='kortexdevlop/kortex-site',headers={Authorization:'Bearer '+env.GITHUB_TOKEN,Accept:'application/vnd.github+json','Content-Type':'application/json'},tag='desktop-staging-v'+descriptor.version;
const api=async(path,method='GET',body)=>{const r=await fetch('https://api.github.com/repos/'+repository+path,{method,headers,body:body===undefined?undefined:JSON.stringify(body),redirect:'error',signal:AbortSignal.timeout(30000)});if(!r.ok)throw Error('GITHUB_RELEASE_'+r.status);return r.json()};
let release;const existing=await fetch('https://api.github.com/repos/'+repository+'/releases/tags/'+tag,{headers,redirect:'error',signal:AbortSignal.timeout(15000)});if(existing.status===404)release=await api('/releases','POST',{tag_name:tag,target_commitish:env.GITHUB_SHA,name:'Kortex Desktop '+descriptor.version+' — Windows staging',body:'Version de test Windows x64 non signée. Installation, démarrage et désinstallation vérifiés. SHA-256 : '+descriptor.sha256,prerelease:true,make_latest:'false'});else if(existing.ok)release=await existing.json();else throw Error('GITHUB_RELEASE_UNAVAILABLE');
async function upload(name,data,type){const old=release.assets.find(a=>a.name===name);if(old){const response=await fetch(old.browser_download_url,{redirect:'follow',signal:AbortSignal.timeout(60000)});if(!response.ok||createHash('sha256').update(new Uint8Array(await response.arrayBuffer())).digest('hex')!==createHash('sha256').update(data).digest('hex'))throw Error('IMMUTABLE_DESKTOP_RELEASE_CONFLICT');return old.browser_download_url}
 const response=await fetch(release.upload_url.replace(/\{.*$/,'')+'?name='+encodeURIComponent(name),{method:'POST',headers:{Authorization:headers.Authorization,'Content-Type':type},body:data,redirect:'error',signal:AbortSignal.timeout(120000)});if(!response.ok)throw Error('DESKTOP_UPLOAD_FAILED_'+response.status);return(await response.json()).browser_download_url}
descriptor.url=await upload(descriptor.filename,bytes,'application/octet-stream');
const statement={descriptor,repository,commit:env.GITHUB_SHA,workflow:env.GITHUB_WORKFLOW_REF},seed=Buffer.from(env.KORTEX_SIGNING_KEY_SEED_BASE64||'','base64');let signature;try{if(seed.length!==32)throw Error('PROTECTED_SIGNING_KEY_REQUIRED');const key=createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),seed]),format:'der',type:'pkcs8'});signature=sign(null,Buffer.from('kortex-desktop-release-v1\n'+canonicalRuntime(statement)+'\n'),key).toString('base64')}finally{seed.fill(0)}
const proof=Buffer.from(JSON.stringify({statement,signing_key_id:env.KORTEX_SIGNING_KEY_ID,signature,keys:JSON.parse(readFileSync('infra/trust/release-staging-keys.json'))},null,2)+'\n');await upload('desktop-release.signed.json',proof,'application/json');
writeFileSync(folder+'/published-desktop.json',JSON.stringify(descriptor,null,2)+'\n');
await publishRuntime({...env,KORTEX_DESKTOP_DESCRIPTOR:folder+'/published-desktop.json',KORTEX_RUNTIME_EXISTING_ONLY:'true'},'staging');
console.log(JSON.stringify({published:true,channel:'staging',release:release.html_url,url:descriptor.url,sha256:descriptor.sha256,code_signed:false}));
