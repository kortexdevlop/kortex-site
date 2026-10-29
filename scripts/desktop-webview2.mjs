import {existsSync,mkdirSync,readFileSync,writeFileSync,copyFileSync,statSync} from 'node:fs';
import {resolve,dirname} from 'node:path';
import {homedir} from 'node:os';
import {createHash} from 'node:crypto';
import {spawnSync} from 'node:child_process';
const publisher='CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US';
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
function signatureDetails(path){
 const env=Object.fromEntries(Object.entries(process.env).filter(([name])=>!/(TOKEN|SECRET|PASSWORD|PEPPER|PRIVATE_KEY|SIGNING_KEY_SEED)/i.test(name)));env.KORTEX_WEBVIEW_VERIFY_PATH=path;
 const r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command',"$s=Get-AuthenticodeSignature -LiteralPath $env:KORTEX_WEBVIEW_VERIFY_PATH; @{status=$s.Status.ToString();subject=$s.SignerCertificate.Subject} | ConvertTo-Json -Compress"],{encoding:'utf8',windowsHide:true,timeout:45000,env});
 try{if(r.status!==0)throw Error();return JSON.parse(r.stdout)}catch{throw Error('WEBVIEW2_SIGNATURE_UNAVAILABLE')}
}
export async function ensureWebView2({manifest=JSON.parse(readFileSync('infra/desktop/webview2-x64.json')),cache=resolve(homedir(),'.dx/tools/MicrosoftEdgeWebView2RuntimeInstaller_x64.exe'),work=resolve('target/desktop-webview2'),fetcher=fetch,signature=signatureDetails}={}){
 const url=new URL(manifest.url);
 if(manifest.schema_version!==1||manifest.architecture!=='x64'||url.protocol!=='https:'||url.hostname!=='msedge.sf.dl.delivery.mp.microsoft.com'||!url.pathname.endsWith('/MicrosoftEdgeWebView2RuntimeInstallerX64.exe')||url.username||url.password||url.search||url.hash||!/^[a-f0-9]{64}$/.test(manifest.sha256)||!Number.isSafeInteger(manifest.bytes)||manifest.bytes<1||manifest.bytes>300*1024*1024||manifest.publisher_subject!==publisher)throw Error('WEBVIEW2_PIN_INVALID');
 mkdirSync(work,{recursive:true});
 const matches=path=>existsSync(path)&&statSync(path).size===manifest.bytes&&digest(readFileSync(path))===manifest.sha256;
 let candidate=cache;
 if(!matches(cache)){
  candidate=resolve(work,manifest.sha256+'.exe');
  if(!matches(candidate)){
   const response=await fetcher(manifest.url,{redirect:'error',signal:AbortSignal.timeout(120000)});if(!response.ok||!response.body)throw Error('WEBVIEW2_DOWNLOAD_FAILED');
   const chunks=[];let size=0;for await(const chunk of response.body){size+=chunk.byteLength;if(size>manifest.bytes)throw Error('WEBVIEW2_DOWNLOAD_SIZE');chunks.push(Buffer.from(chunk))}
   const bytes=Buffer.concat(chunks);if(bytes.length!==manifest.bytes||digest(bytes)!==manifest.sha256)throw Error('WEBVIEW2_DOWNLOAD_HASH');writeFileSync(candidate,bytes);
  }
 }
 const signed=await signature(candidate);if(signed.status!=='Valid'||signed.subject!==publisher)throw Error('WEBVIEW2_SIGNATURE_INVALID');
 if(candidate!==cache){
  if(existsSync(cache)){const oldHash=digest(readFileSync(cache));copyFileSync(cache,resolve(work,'previous-'+oldHash+'.exe'))}
  mkdirSync(dirname(cache),{recursive:true});copyFileSync(candidate,cache);
 }
 return{architecture:'x64',installer_version:manifest.installer_version,sha256:manifest.sha256,bytes:manifest.bytes,publisher_verified:true};
}
