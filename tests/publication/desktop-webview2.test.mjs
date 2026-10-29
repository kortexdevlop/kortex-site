import {test} from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync,mkdirSync,writeFileSync,readFileSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {ensureWebView2} from '../../scripts/desktop-webview2.mjs';
const bytes=Buffer.from('MZ-unit-x64-fixture'),sha256=createHash('sha256').update(bytes).digest('hex'),publisher_subject='CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US';
const manifest={schema_version:1,architecture:'x64',installer_version:'unit-only',url:'https://msedge.sf.dl.delivery.mp.microsoft.com/fixture/MicrosoftEdgeWebView2RuntimeInstallerX64.exe',sha256,bytes:bytes.length,publisher_subject};
function paths(){mkdirSync('target',{recursive:true});const root=mkdtempSync(resolve('target/webview2-test-'));return{cache:resolve(root,'cache.exe'),work:resolve(root,'work')}}
const signature=async()=>({status:'Valid',subject:publisher_subject});
test('replaces a mislabeled cached x86 payload only after hash and publisher verification, preserving its bytes',async()=>{
 const p=paths(),old=Buffer.from('old-x86');writeFileSync(p.cache,old);let requests=0;
 const fetcher=async(_url,init)=>{requests++;assert.equal(init.redirect,'error');return new Response(bytes)};
 const receipt=await ensureWebView2({...p,manifest,fetcher,signature});assert.equal(receipt.architecture,'x64');assert.deepEqual(readFileSync(p.cache),bytes);const oldHash=createHash('sha256').update(old).digest('hex');assert.deepEqual(readFileSync(resolve(p.work,'previous-'+oldHash+'.exe')),old);
 await ensureWebView2({...p,manifest,fetcher,signature});assert.equal(requests,1);
});
test('wrong architecture, altered bytes and untrusted publisher never overwrite the cache',async()=>{
 const p=paths(),old=Buffer.from('old');writeFileSync(p.cache,old);
 await assert.rejects(ensureWebView2({...p,manifest:{...manifest,architecture:'x86'},signature}),/PIN_INVALID/);
 await assert.rejects(ensureWebView2({...p,manifest,signature,fetcher:async()=>new Response(Buffer.alloc(bytes.length,1))}),/DOWNLOAD_HASH/);
 await assert.rejects(ensureWebView2({...p,manifest,fetcher:async()=>new Response(bytes),signature:async()=>({status:'NotSigned',subject:''})}),/SIGNATURE_INVALID/);
 assert.deepEqual(readFileSync(p.cache),old);assert.equal(existsSync(resolve(p.work,'previous-'+createHash('sha256').update(old).digest('hex')+'.exe')),false);
});
