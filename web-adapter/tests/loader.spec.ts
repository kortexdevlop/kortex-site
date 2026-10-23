import {test,expect} from 'playwright/test';
import {serve} from '../../tests/web/harness';
import {generateKeyPairSync,sign,createHash} from 'node:crypto';
let server:Awaited<ReturnType<typeof serve>>;
test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>server.close());
test('trust store : signature exacte, altération, clé inconnue, révocation et expiration',async({page})=>{
 const keys=generateKeyPairSync('ed25519'),bytes=Buffer.from('signed archive fixture'),sha256=createHash('sha256').update(bytes).digest('hex');
 const proof={tool_id:'test-tool',version:'1.0.0',abi_version:'1.0',sha256,signing_key_id:'test',signature:sign(null,Buffer.from(`kortex-artifact-v1\ntest-tool\n1.0.0\n1.0\n${sha256}\n`),keys.privateKey).toString('base64')};
 await page.goto(server.origin);
 const results=await page.evaluate(async({proof,bytes,key})=>{
  const {TrustStore}=await import('/adapter/trust.js' as string);const store=new TrustStore([{id:'test',public_key_base64:key,not_before:0,expires:200,revoked:false}]);const data=new Uint8Array(bytes);const results=[];
  await store.verify(data,proof,100);results.push('VALID');
  for(const [value,p,time] of [[new Uint8Array([1]),proof,100],[data,{...proof,signing_key_id:'unknown'},100],[data,proof,200],[data,{...proof,abi_version:'2.0'},100]] as const){try{await store.verify(value,p,time);results.push('ACCEPT')}catch(e){results.push((e as Error).message)}}
  store.revoke('test');try{await store.verify(data,proof,100)}catch(e){results.push((e as Error).message)}return results;
 },{proof,bytes:Array.from(bytes),key:keys.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64')});
 expect(results).toEqual(['VALID','INTEGRITY_ERROR','UNTRUSTED_KEY','UNTRUSTED_KEY','UNSUPPORTED_ABI','UNTRUSTED_KEY']);
});

test('archive signée, ZIP hostile, cache corrompu et MIME refusés avant exécution',async({page})=>{
 const {prepare}=await import('./helpers');await page.goto(server.origin);await prepare(page);
 const fixture=await page.evaluate(async()=>{
  const t=(window as any).adapterTest;const {crc32}=await import('/adapter/archive.js' as string);const {TrustStore}=await import('/adapter/host.js' as string);
  function zip(entries:[string,Uint8Array][],bomb=false){const local:number[]=[],central:number[]=[];const put=(arr:number[],n:number,size:number)=>{for(let i=0;i<size;i++)arr.push((n>>>8*i)&255)};
   for(const [name,data] of entries){const n=new TextEncoder().encode(name),offset=local.length,crc=crc32(data),length=bomb?21*1024*1024:data.length;
    put(local,0x04034b50,4);put(local,20,2);put(local,0x800,2);put(local,0,2);put(local,0,4);put(local,crc,4);put(local,data.length,4);put(local,length,4);put(local,n.length,2);put(local,0,2);local.push(...n);for(const b of data)local.push(b);
    put(central,0x02014b50,4);put(central,20,2);put(central,20,2);put(central,0x800,2);put(central,0,2);put(central,0,4);put(central,crc,4);put(central,data.length,4);put(central,length,4);put(central,n.length,2);put(central,0,2);put(central,0,2);put(central,0,2);put(central,0,2);put(central,0,4);put(central,offset,4);central.push(...n);
   }
   const end:number[]=[];put(end,0x06054b50,4);put(end,0,4);put(end,entries.length,2);put(end,entries.length,2);put(end,central.length,4);put(end,local.length,4);put(end,0,2);return new Uint8Array([...local,...central,...end]);
  }
  const encoder=new TextEncoder(),module=new Uint8Array(await(await fetch('/fixtures/guest.wasm')).arrayBuffer());const entries:[string,Uint8Array][]=[['manifest.json',encoder.encode(JSON.stringify(t.manifest))],['contract.json',encoder.encode(JSON.stringify(t.contract))],['module.wasm',module],['assets/icon.svg',encoder.encode('<svg xmlns="http://www.w3.org/2000/svg"/>')]];
  const trust=new TrustStore([{id:'test',public_key_base64:btoa(String.fromCharCode(...t.publicKey)),not_before:0,expires:Date.now()+60000,revoked:false}]);
  async function proof(bytes:Uint8Array){const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');return {tool_id:t.manifest.id,version:t.manifest.version,abi_version:'1.0',sha256,signing_key_id:'test',signature:await (window as any).signTestArtifact(Array.from(encoder.encode(`kortex-artifact-v1\n${t.manifest.id}\n${t.manifest.version}\n1.0\n${sha256}\n`)))};}
  const bytes=zip(entries),descriptor=await proof(bytes);const tool=await t.api.VerifiedTool.verifyArchive(bytes,descriptor,trust);const bridge=await t.api.startVerifiedTool(tool);const reply=await bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind:'action.invoke',payload:{action_id:'multiply',input:{a:3,b:7}}});bridge.stop();
  const errors=[];const tampered=bytes.slice();tampered[40]^=1;try{await t.api.VerifiedTool.verifyArchive(tampered,descriptor,trust)}catch(e){errors.push((e as Error).message)}
  for(const bad of [zip([['../escape',encoder.encode('x')],...entries]),zip(entries,true)]){try{await t.api.VerifiedTool.verifyArchive(bad,await proof(bad),trust)}catch(e){errors.push((e as Error).message)}}
  (window as any).archiveTest={bytes,descriptor,trust};return {bytes:Array.from(bytes),descriptor,errors,result:reply.data.result};
 });
 expect(fixture.result).toBe(21);expect(fixture.errors).toEqual(['INTEGRITY_ERROR','INVALID_ARCHIVE','ARCHIVE_LIMIT']);
 await page.route(server.origin+'/tool',route=>route.fulfill({contentType:'application/zip',body:Buffer.from(fixture.bytes)}));
 const outcome=await page.evaluate(async(origin)=>{const a=(window as any).archiveTest;const {ToolLoader}=await import('/adapter/host.js' as string);const loader=new ToolLoader(a.trust,origin);const d={...a.descriptor,url:origin+'/tool'};await loader.loadTool(d);const cache=await caches.open('kortex-artifacts-v1');const keys=await cache.keys();await cache.put(keys[0],new Response(new Uint8Array([1,2,3])));let code='';try{await loader.loadTool(d)}catch(e){code=(e as Error).message}const remaining=(await cache.keys()).length;return {code,remaining}},server.origin);
 expect(outcome).toEqual({code:'INTEGRITY_ERROR',remaining:0});
 await page.route(server.origin+'/wrong',route=>route.fulfill({contentType:'text/html',body:'invalid'}));
 expect(await page.evaluate(async(origin)=>{const a=(window as any).archiveTest;const {ToolLoader}=await import('/adapter/host.js' as string);try{await new ToolLoader(a.trust,origin).loadTool({...a.descriptor,url:origin+'/wrong'});return 'ACCEPT'}catch(e){return (e as Error).message}},server.origin)).toBe('INVALID_DOWNLOAD');
});
