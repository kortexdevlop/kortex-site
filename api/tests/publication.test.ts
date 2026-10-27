import {env,exports} from 'cloudflare:workers';
import {applyD1Migrations} from 'cloudflare:test';
import {beforeAll,describe,it,expect} from 'vitest';
import {handle} from '../src/index';
import {extractArchive,crc32} from '../../web-adapter/src/archive';
import {canonical,hex} from '../src/lib/admin-error';
import type {Publication} from '../src/services/publication';
declare global {namespace Cloudflare {interface Env {TEST_AUTH_PRIVATE_JWK:JsonWebKey;TEST_ARCHIVE_BASE64:string}}}
const encode=(b:Uint8Array)=>btoa(String.fromCharCode(...b)).replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'');
const encoder=new TextEncoder();
let privateKey:CryptoKey,files:Map<string,Uint8Array>;
beforeAll(async()=>{
 await applyD1Migrations(env.DB,env.TEST_MIGRATIONS);
 privateKey=await crypto.subtle.importKey('jwk',env.TEST_AUTH_PRIVATE_JWK,{name:'Ed25519'},false,['sign']);
 files=await extractArchive(Uint8Array.from(atob(env.TEST_ARCHIVE_BASE64),c=>c.charCodeAt(0)));
 const publicRaw=Uint8Array.from(atob(String(env.TEST_AUTH_PRIVATE_JWK.x).replaceAll('-','+').replaceAll('_','/')),c=>c.charCodeAt(0));
 await env.DB.prepare("INSERT INTO signing_keys VALUES('ephemeral-api-test',?,1,0,9999999999999,0)").bind(btoa(String.fromCharCode(...publicRaw))).run();
});
async function token(overrides:Record<string,unknown>={},headerOverrides:Record<string,unknown>={}) {
 const p=JSON.parse(env.ADMIN_AUTH_POLICY!),now=Math.floor(Date.now()/1000),head=encode(encoder.encode(JSON.stringify({alg:'EdDSA',kid:'ephemeral-api-test',typ:'JWT',...headerOverrides})));
 const payload=encode(encoder.encode(JSON.stringify({iss:p.issuer,aud:p.audience,sub:p.subject,repository:p.repository,repository_id:p.repository_id,ref:p.ref,workflow_ref:p.workflow_ref,iat:now,exp:now+300,jti:crypto.randomUUID(),...overrides})));
 const message=head+'.'+payload;return message+'.'+encode(new Uint8Array(await crypto.subtle.sign('Ed25519',privateKey,encoder.encode(message))));
}
export function zipFiles(entries:Map<string,Uint8Array>):Uint8Array {
 const parts:Uint8Array[]=[],central:Uint8Array[]=[];let offset=0,centralLength=0;
 for(const [name,data] of [...entries].sort(([a],[b])=>a<b?-1:1)){
  const n=encoder.encode(name),local=new Uint8Array(30+n.length),l=new DataView(local.buffer),crc=crc32(data);
  l.setUint32(0,0x04034b50,true);l.setUint16(4,20,true);l.setUint16(12,33,true);l.setUint32(14,crc,true);l.setUint32(18,data.length,true);l.setUint32(22,data.length,true);l.setUint16(26,n.length,true);local.set(n,30);
  const record=new Uint8Array(46+n.length),r=new DataView(record.buffer);r.setUint32(0,0x02014b50,true);r.setUint16(4,0x0314,true);r.setUint16(6,20,true);r.setUint16(14,33,true);r.setUint32(16,crc,true);r.setUint32(20,data.length,true);r.setUint32(24,data.length,true);r.setUint16(28,n.length,true);r.setUint32(38,0x81a40000,true);r.setUint32(42,offset,true);record.set(n,46);
  parts.push(local,data);offset+=local.length+data.length;central.push(record);centralLength+=record.length;
 }
 const end=new Uint8Array(22),e=new DataView(end.buffer);e.setUint32(0,0x06054b50,true);e.setUint16(8,entries.size,true);e.setUint16(10,entries.size,true);e.setUint32(12,centralLength,true);e.setUint32(16,offset,true);
 const result=new Uint8Array(offset+centralLength+22);let at=0;for(const p of [...parts,...central,end]){result.set(p,at);at+=p.length}return result;
}
async function staged(id:string,version='1.0.0') {
 const manifest={...env.TEST_MANIFEST,id,version} as Publication['manifest'],contract={...env.TEST_CONTRACT,tool_id:id} as unknown as Publication['contract'];
 const contents=new Map(files);contents.set('manifest.json',encoder.encode(canonical(manifest)));contents.set('contract.json',encoder.encode(canonical(contract)));
 const bytes=zipFiles(contents),sha256=hex(await crypto.subtle.digest('SHA-256',bytes));
 const descriptor={schema_version:1 as const,tool_id:id,version,abi_version:'1.0',sha256,signing_key_id:'ephemeral-api-test',signature:''};
 descriptor.signature=btoa(String.fromCharCode(...new Uint8Array(await crypto.subtle.sign('Ed25519',privateKey,encoder.encode(`kortex-artifact-v1\n${id}\n${version}\n1.0\n${sha256}\n`)))));
 const request:Publication={manifest,contract,descriptor,r2_key:`tools/${id}/${version}/${sha256}.tool`,request_id:crypto.randomUUID()};await env.TOOLS_BUCKET.put(request.r2_key,bytes,{onlyIf:{etagDoesNotMatch:'*'}});return {request,bytes};
}
async function post(path:string,body:unknown,bearer?:string) {return exports.default.fetch(new Request('https://api.test/api/v1/admin/'+path,{method:'POST',headers:{Authorization:'Bearer '+(bearer??await token()),'Content-Type':'application/json'},body:JSON.stringify(body)}))}
const pub=(body:unknown,bearer?:string)=>post('publications',body,bearer);
const get=(id:string)=>exports.default.fetch(new Request('https://api.test/api/v1/tools/'+id));
describe('private verified publication and mutation transactions',()=>{
 it('requires short-lived signed claims and validates every trust constraint',async()=>{
  for(const claims of [{aud:'wrong'},{iss:'https://evil.test'},{sub:'other'},{repository:'other/kortex'},{repository_id:'2'},{ref:'refs/heads/pull'},{workflow_ref:'other'},{exp:1},{iat:Math.floor(Date.now()/1000)+90},{exp:Math.floor(Date.now()/1000)+900},{nbf:Math.floor(Date.now()/1000)+100}])expect((await pub({},await token(claims))).status).toBe(401);
  for(const alg of ['none','HS256','RS256'])expect((await pub({},await token({},{alg}))).status).toBe(401);
  expect((await pub({},'bad')).status).toBe(401);
  const signed=await token();expect((await pub({},signed.slice(0,-4)+'abcd')).status).toBe(401);
  const noKey=await handle(new Request('https://api.test/api/v1/admin/publications',{method:'POST'}),{...env,ADMIN_AUTH_POLICY:undefined});expect(noKey.status).toBe(503);
  const p=JSON.parse(env.ADMIN_AUTH_POLICY!);p.actions=['rollback'];const forbidden=await handle(new Request('https://api.test/api/v1/admin/publications',{method:'POST',headers:{Authorization:'Bearer '+await token(),'Content-Type':'application/json'},body:'{}'}),{...env,ADMIN_AUTH_POLICY:JSON.stringify(p)});expect(forbidden.status).toBe(403);
 });
 it('publishes real verified R2 bytes atomically, serves descriptor/actions/download and replays without repointing',async()=>{
  const a=await staged('publish-real');expect((await get('publish-real')).status).toBe(404);const r=await pub(a.request);expect(r.status).toBe(200);expect(await r.json()).toMatchObject({status:'published',version:'1.0.0'});
  const detail=await get('publish-real');expect(detail.status).toBe(200);expect((await detail.json() as {descriptor:unknown}).descriptor).toEqual(a.request.descriptor);
  const download=await exports.default.fetch(new Request('https://api.test/api/v1/downloads/publish-real/1.0.0'));expect(new Uint8Array(await download.arrayBuffer())).toEqual(a.bytes);
  expect((await pub(a.request)).status).toBe(200);expect(await env.DB.prepare('SELECT count(*) n FROM publication_jobs WHERE tool_id=?').bind('publish-real').first('n')).toBe(1);
  expect((await pub({...a.request,manifest:{...a.request.manifest,name:'Changed'}})).status).toBe(409);
 });
 it('rejects hash/signature/metadata/ABI/permissions/ZIP tampering before anything becomes public',async()=>{
  const a=await staged('publish-corrupt');const changed=a.bytes.slice();changed[70]^=1;await env.TOOLS_BUCKET.put(a.request.r2_key,changed);expect((await pub(a.request)).status).toBe(422);expect((await get('publish-corrupt')).status).toBe(404);
  await env.TOOLS_BUCKET.put(a.request.r2_key,a.bytes);
  expect((await pub({...a.request,descriptor:{...a.request.descriptor,signature:'A'.repeat(86)+'=='}})).status).toBe(422);
  expect((await pub({...a.request,manifest:{...a.request.manifest,name:'Metadata does not match ZIP'}})).status).toBe(422);
  expect((await pub({...a.request,manifest:{...a.request.manifest,abi_version:'2.0'}})).status).toBe(400);
  expect((await pub({...a.request,contract:{...a.request.contract,actions:a.request.contract.actions.map(x=>({...x,permissions:['network.fetch']}))}})).status).toBe(422);
  expect(await env.DB.prepare('SELECT count(*) n FROM tool_versions WHERE tool_id=?').bind('publish-corrupt').first('n')).toBe(0);
 });
 it('serializes a version race, preserves the winning bytes and journals only committed operations',async()=>{
  const a=await staged('publish-race'),b={...a.request,request_id:crypto.randomUUID()};
  const results=await Promise.all([pub(a.request),pub(b)]);expect(results.map(r=>r.status).sort()).toEqual([200,409]);
  expect(await env.DB.prepare('SELECT count(*) n FROM admin_operations WHERE tool_id=?').bind('publish-race').first('n')).toBe(1);
 });
 it('keeps an in-flight verified publication invisible until the atomic D1 batch',async()=>{
  const a=await staged('publish-inflight');let release!:()=>void,entered!:()=>void;const wait=new Promise<void>(r=>release=r),ready=new Promise<void>(r=>entered=r);
  const bucket={head:env.TOOLS_BUCKET.head.bind(env.TOOLS_BUCKET),put:env.TOOLS_BUCKET.put.bind(env.TOOLS_BUCKET),async get(key:string,options:R2GetOptions){entered();await wait;return env.TOOLS_BUCKET.get(key,options)}};
  const pending=handle(new Request('https://api.test/api/v1/admin/publications',{method:'POST',headers:{Authorization:'Bearer '+await token(),'Content-Type':'application/json'},body:JSON.stringify(a.request)}),{...env,TOOLS_BUCKET:bucket});
  await ready;expect((await get('publish-inflight')).status).toBe(404);expect(await env.DB.prepare('SELECT count(*) n FROM admin_operations WHERE tool_id=?').bind('publish-inflight').first('n')).toBe(0);release();expect((await pending).status).toBe(200);
 });
 it('rolls back to a healthy version with compare-and-swap and revokes atomically with replay safety',async()=>{
  const one=await staged('publish-rollback'),two=await staged('publish-rollback','1.1.0');expect((await pub(one.request)).status).toBe(200);expect((await pub(two.request)).status).toBe(200);
  const body={request_id:crypto.randomUUID(),version:'1.0.0',expected_current_version:'1.1.0'};expect((await post('tools/publish-rollback/rollback',body)).status).toBe(200);
  expect((await (await get('publish-rollback')).json() as {tool:{version:string}}).tool.version).toBe('1.0.0');
  expect((await pub(two.request)).status).toBe(200);expect((await (await get('publish-rollback')).json() as {tool:{version:string}}).tool.version).toBe('1.0.0');
  expect((await post('tools/publish-rollback/rollback',{...body,request_id:crypto.randomUUID()})).status).toBe(409);expect((await post('tools/publish-rollback/rollback',body)).status).toBe(200);
  const revoked={request_id:crypto.randomUUID(),reason:'TEST compromised release'};expect((await post('artifacts/'+one.request.descriptor.sha256+'/revoke',revoked)).status).toBe(200);expect((await get('publish-rollback')).status).toBe(404);
  expect((await exports.default.fetch(new Request('https://api.test/api/v1/downloads/publish-rollback/1.0.0'))).status).toBe(410);
  expect((await post('artifacts/'+one.request.descriptor.sha256+'/revoke',revoked)).status).toBe(200);
  expect((await pub(one.request)).status).toBe(200);expect((await get('publish-rollback')).status).toBe(404);
  expect((await post('tools/publish-rollback/rollback',{request_id:crypto.randomUUID(),version:'1.0.0',expected_current_version:'1.1.0'})).status).toBe(409);
 });
 it('enforces the streamed 256KiB request cap and strict mutation bodies',async()=>{
  expect((await pub({unknown:'x'.repeat(262145)})).status).toBe(413);expect((await pub({})).status).toBe(400);
  expect((await post('tools/publish-real/rollback',{request_id:crypto.randomUUID(),version:'1.0.0'})).status).toBe(400);
 });
});
