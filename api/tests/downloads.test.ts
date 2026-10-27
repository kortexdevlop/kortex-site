import {env,exports} from 'cloudflare:workers';
import {describe,it,expect} from 'vitest';
import {seedArtifact} from './helpers';
import {checkedBody} from '../src/services/storage';
const call=(id:string,version='1.0.0',init?:RequestInit)=>exports.default.fetch(new Request(`https://api.test/api/v1/downloads/${id}/${version}`,init));
describe('private R2 download path',()=>{
 it('returns exact local R2 bytes, public descriptor/digest/link, HEAD and conditional cache',async()=>{
  const bytes=new Uint8Array([80,75,1,2,3,4,5]),a=await seedArtifact('download-exact',bytes);
  const r=await call('download-exact');expect(r.status).toBe(200);expect(new Uint8Array(await r.arrayBuffer())).toEqual(bytes);
  expect(r.headers.get('content-type')).toBe('application/vnd.kortex.tool+zip');expect(r.headers.get('x-content-type-options')).toBe('nosniff');expect(r.headers.get('x-kortex-sha256')).toBe(a.sha256);
  expect(r.headers.get('cache-control')).toBe('public, max-age=0, must-revalidate');
  const etag=r.headers.get('etag')!;const conditional=await call('download-exact','1.0.0',{headers:{'If-None-Match':etag}});expect(conditional.status).toBe(304);expect(await conditional.text()).toBe('');
  const head=await call('download-exact','1.0.0',{method:'HEAD'});expect(head.status).toBe(200);expect(await head.text()).toBe('');expect(head.headers.get('content-length')).toBe(String(bytes.length));
  const detail=await exports.default.fetch(new Request('https://api.test/api/v1/tools/download-exact'));expect(detail.headers.get('link')).toContain('/api/v1/downloads/download-exact/1.0.0');
  expect((await detail.json() as {descriptor:unknown}).descriptor).toEqual(a.descriptor);
 });
 it('refuses missing objects, unpublished versions, replacement and revoked artifacts even with an old ETag',async()=>{
  expect((await call('download-missing')).status).toBe(404);
  const missing=await seedArtifact('download-object-missing',new Uint8Array([1]));await env.TOOLS_BUCKET.delete(missing.key);expect((await call('download-object-missing')).status).toBe(404);
  const changed=await seedArtifact('download-replaced',new Uint8Array([1,2]));await env.TOOLS_BUCKET.put(changed.key,new Uint8Array([3,4]));expect((await call('download-replaced')).status).toBe(503);
  const revoked=await seedArtifact('download-revoked',new Uint8Array([7]));
  await env.DB.batch([env.DB.prepare("UPDATE tools SET status='archived',published_version=NULL WHERE id='download-revoked'"),env.DB.prepare('INSERT INTO revocations(request_id,tool_id,version,reason,created_at) VALUES(?,?,?,?,?)').bind(crypto.randomUUID(),'download-revoked','1.0.0','TEST revocation',Date.now())]);
  expect((await call('download-revoked','1.0.0',{headers:{'If-None-Match':'"'+revoked.sha256+'"'}})).status).toBe(410);
  await seedArtifact('download-archived',new Uint8Array([1]));await env.DB.prepare("UPDATE tools SET status='archived' WHERE id='download-archived'").run();expect((await call('download-archived')).status).toBe(404);
 });
 it('streams a large generated R2 body lazily without arrayBuffer or concatenation',async()=>{
  let pulled=0;const size=24*1024*1024,chunkSize=65536;
  const bucket={async get(){return {etag:'TEST',size,body:new ReadableStream<Uint8Array>({pull(controller){if(pulled===size){controller.close();return}pulled+=chunkSize;controller.enqueue(new Uint8Array(chunkSize))}})}}} as unknown as R2Bucket;
  const stream=await checkedBody(bucket,{r2_key:'test',byte_length:size,head_etag:'TEST',sha256:'0'.repeat(64)});expect(pulled).toBeLessThan(size);
  let read=0;const reader=stream!.getReader();for(;;){const r=await reader.read();if(r.done)break;read+=r.value.length;expect(r.value.length).toBe(chunkSize)}expect(read).toBe(size);
 });
 it('validates route and method',async()=>{expect((await call('Bad_ID')).status).toBe(400);expect((await call('download-exact','bad')).status).toBe(400);expect((await call('download-exact','1.0.0',{method:'POST'})).status).toBe(405)});
});
