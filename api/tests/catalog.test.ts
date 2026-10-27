import {env,exports} from 'cloudflare:workers';
import {applyD1Migrations,type D1Migration} from 'cloudflare:test';
import {beforeAll,describe,it,expect} from 'vitest';
import {handle} from '../src/index';
import {cursorKey,signCursor} from '../src/lib/cursor';
import {query} from '../src/services/catalog';
declare global {namespace Cloudflare {interface Env {TEST_MIGRATIONS:D1Migration[];TEST_MANIFEST:Record<string,unknown>;TEST_CONTRACT:{actions:Record<string,unknown>[]};TEST_DESCRIPTOR:Record<string,unknown>}}}
type Page={items:{id:string;name:string;category:string;tags:string[];published_at:number}[];next_cursor:string|null};
const fetchApi=(path='',init?:RequestInit)=>exports.default.fetch(new Request('https://api.test/api/v1/tools'+path,init));
const testId='tool-00001';
beforeAll(async()=>{
 await applyD1Migrations(env.DB,env.TEST_MIGRATIONS);
 await env.DB.prepare("INSERT INTO categories VALUES('alpha','Alpha'),('bravo','Bravo')").run();
 await env.DB.prepare(`WITH RECURSIVE seq(n) AS (VALUES(1) UNION ALL SELECT n+1 FROM seq WHERE n<10002)
 INSERT INTO tools(id,name,name_norm,summary,category_id,created_at,updated_at)
 SELECT printf('tool-%05d',n),printf('Tool %03d',n%100),printf('tool %03d',n%100),'Searchable catalogue',CASE n%2 WHEN 0 THEN 'alpha' ELSE 'bravo' END,1,1 FROM seq`).run();
 await env.DB.prepare(`INSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,created_at)
 SELECT id,'1.0.0',printf('%064x',1),'1.0','1.0',json_set(?,'$.id',id,'$.name',name,'$.summary',summary,'$.category',category_id,'$.tags',json_array(CASE category_id WHEN 'alpha' THEN 'even' ELSE 'odd' END)),json_set(?,'$.tool_id',id),1 FROM tools`).bind(JSON.stringify(env.TEST_MANIFEST),JSON.stringify(env.TEST_CONTRACT)).run();
 // Real D1 state transitions, with synthetic TEST attestations only. No cryptographic/R2 verification claim.
 await env.DB.prepare("INSERT INTO signing_keys VALUES('catalog-test',?,1,0,9999999999,0)").bind('A'.repeat(43)+'=').run();
 await env.DB.prepare(`INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at)
 SELECT tool_id,version,sha256,'tools/'||tool_id||'/'||version||'/'||sha256||'.tool',100,'catalog-test',json_set(?,'$.tool_id',tool_id,'$.sha256',sha256,'$.signing_key_id','catalog-test'),1 FROM tool_versions`).bind(JSON.stringify(env.TEST_DESCRIPTOR)).run();
 await env.DB.prepare("UPDATE artifacts SET head_byte_length=100,head_etag='TEST',read_byte_length=100,read_sha256=sha256,signature_verified_at=2,r2_verified_at=2").run();
 await env.DB.prepare(`INSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at)
 SELECT printf('93000000-0000-4000-8000-%012d',CAST(substr(tool_id,6) AS INTEGER)),sha256,tool_id,version,2,2 FROM artifacts WHERE tool_id!='tool-10002'`).run();
 await env.DB.prepare("UPDATE publication_jobs SET status='validated',updated_at=3").run();
 await env.DB.prepare("UPDATE publication_jobs SET status='published',updated_at=4").run();
 await env.DB.prepare("UPDATE tool_versions SET status='published' WHERE tool_id!='tool-10002'").run();
 await env.DB.prepare("UPDATE tools SET status='published',published_version='1.0.0',published_at=CAST(substr(id,6) AS INTEGER)%7,updated_at=4 WHERE id<='tool-10000'").run();
 await env.DB.prepare("UPDATE tool_versions SET status='revoked' WHERE tool_id='tool-10001'").run();
 // A newer unpublished version must never supply the current actions.
 await env.DB.prepare(`INSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,created_at)
 SELECT tool_id,'2.0.0',printf('%064x',2),'1.0','1.0',json_set(manifest_json,'$.version','2.0.0'),json_set(contract_json,'$.actions',json_array()),5 FROM tool_versions WHERE tool_id=?`).bind(testId).run();
});
describe('catalogue API V1 in real D1/workerd',()=>{
 for(const sort of ['recent','name'])it(`walks 10,000 published entries without holes or duplicates (${sort}, including ties)`,async()=>{
  const ids:string[]=[],keys:(number|string)[]=[];let cursor:string|null=null,pages=0;
  do {
   const response=await fetchApi('?limit=100&sort='+sort+(cursor?'&cursor='+encodeURIComponent(cursor):''));expect(response.status).toBe(200);
   const page=await response.json() as Page;expect(page.items.length).toBeGreaterThan(0);expect(page.items.length).toBeLessThanOrEqual(100);
   for(const item of page.items){ids.push(item.id);keys.push(sort==='name'?item.name.toLowerCase():item.published_at)}
   cursor=page.next_cursor;expect(++pages).toBeLessThanOrEqual(100);
  }while(cursor);
  expect(ids.length).toBe(10000);expect(new Set(ids).size).toBe(10000);
  expect([...ids].sort()).toEqual(Array.from({length:10000},(_,i)=>'tool-'+String(i+1).padStart(5,'0')));
  for(let i=1;i<ids.length;i++){if(keys[i]===keys[i-1])expect(sort==='name'?ids[i]>ids[i-1]:ids[i]<ids[i-1]).toBe(true);else expect(sort==='name'?keys[i]>keys[i-1]:keys[i]<keys[i-1]).toBe(true)}
 });
 it('combines literal search, category and exact tag, with no SQL wildcard/injection interpretation',async()=>{
  const response=await fetchApi('?q=TOOL%20002&category=alpha&tag=even&limit=100');const page=await response.json() as Page;
  expect(response.status).toBe(200);expect(page.items.length).toBe(100);expect(page.next_cursor).toBeNull();
  expect(page.items.every(t=>t.name==='Tool 002'&&t.category==='alpha'&&t.tags.includes('even'))).toBe(true);
  for(const q of ['%',"' OR 1=1 --",'_']){const r=await fetchApi('?q='+encodeURIComponent(q));expect((await r.json() as Page).items).toEqual([])}
  expect((await (await fetchApi('?q=searchable&category=missing')).json() as Page).items).toEqual([]);
 });
 it('serves detail and actions only from the currently published version',async()=>{
  const r=await fetchApi('/'+testId);expect(r.status).toBe(200);const value=await r.json() as {tool:{id:string;version:string};manifest:{id:string};descriptor:{tool_id:string}};
  expect(value.tool).toMatchObject({id:testId,version:'1.0.0'});expect(value.manifest.id).toBe(testId);expect(value.descriptor.tool_id).toBe(testId);
  const a=await fetchApi('/'+testId+'/actions');expect(a.status).toBe(200);expect(await a.json()).toEqual({tool_id:testId,version:'1.0.0',actions:env.TEST_CONTRACT.actions});
  for(const id of ['tool-10001','tool-10002','absent'])for(const suffix of ['','/actions'])expect((await fetchApi('/'+id+suffix)).status).toBe(404);
 });
 it('rejects malformed, duplicate, unbounded and unknown queries',async()=>{
  for(const suffix of ['?limit=0','?limit=101','?limit=1.2','?limit=01','?limit=','?limit=1&limit=2','?sort=random','?status=draft','?q='+ 'x'.repeat(201),'?category=','?category=UPPER','?tag=','?tag='+ 'x'.repeat(41),'?q=%00','?cursor=','?cursor='+ 'x'.repeat(4097),'/../tools/Bad_ID','/'+testId+'?limit=2']){
   const r=await fetchApi(suffix);expect(r.status,suffix.slice(0,80)).toBe(400);expect(r.headers.get('cache-control')).toBe('no-store');
  }
 });
 it('authenticates cursors and binds them to filters, ordering and limit',async()=>{
  const page=await (await fetchApi('?limit=1')).json() as Page;const cursor=page.next_cursor!;
  for(const c of ['x',cursor+'.extra',cursor.slice(0,-3)+'abc',cursor+'=',cursor.split('.')[0]+'.'+btoa('x'.repeat(32))])expect((await fetchApi('?limit=1&cursor='+encodeURIComponent(c))).status).toBe(400);
  for(const query of ['limit=2','limit=1&sort=name','limit=1&q=tool','limit=1&category=alpha','limit=1&tag=even'])expect((await fetchApi('?'+query+'&cursor='+encodeURIComponent(cursor))).status).toBe(400);
  const scope=query(new URLSearchParams('limit=1')).scope;
  const signedWrongType=await signCursor(scope,{id:testId,key:'wrong-type'},await cursorKey(env.CURSOR_SIGNING_KEY));
  expect((await fetchApi('?limit=1&cursor='+encodeURIComponent(signedWrongType))).status).toBe(400);
  const wrongKey=await signCursor(scope,{id:testId,key:1},await cursorKey('94'.repeat(32)));
  expect((await fetchApi('?limit=1&cursor='+encodeURIComponent(wrongKey))).status).toBe(400);
 });
 it('supports conditional GET/HEAD and exposes ETag through exact-origin CORS',async()=>{
  for(const path of ['?limit=1','/'+testId,'/'+testId+'/actions']){
   const first=await fetchApi(path,{headers:{Origin:env.PUBLIC_WEB_ORIGIN}}),etag=first.headers.get('etag')!;expect(etag).toMatch(/^"[0-9a-f]{64}"$/);
   expect(first.headers.get('access-control-expose-headers')).toContain('ETag');expect(first.headers.get('cache-control')).toContain('must-revalidate');
   const unchanged=await fetchApi(path,{headers:{'If-None-Match':'"other", W/'+etag,Origin:env.PUBLIC_WEB_ORIGIN}});expect(unchanged.status).toBe(304);expect(await unchanged.text()).toBe('');expect(unchanged.headers.get('etag')).toBe(etag);expect(unchanged.headers.get('access-control-allow-origin')).toBe(env.PUBLIC_WEB_ORIGIN);
   const head=await fetchApi(path,{method:'HEAD'});expect(head.status).toBe(200);expect(await head.text()).toBe('');expect(head.headers.get('etag')).toBe(etag);
   expect((await fetchApi(path,{method:'POST'})).status).toBe(405);
  }
  const preflight=await fetchApi('',{method:'OPTIONS',headers:{Origin:env.PUBLIC_WEB_ORIGIN,'Access-Control-Request-Method':'GET','Access-Control-Request-Headers':'If-None-Match'}});expect(preflight.status).toBe(204);
 });
 it('fails closed on missing signing secret/database errors without exposing configuration',async()=>{
  for(const overrides of [{CURSOR_SIGNING_KEY:undefined},{CURSOR_SIGNING_KEY:'TEST_SENTINEL'},{DB:{prepare(){throw Error('TEST_SQL_SENTINEL')},batch(){throw Error('TEST_SQL_SENTINEL')}}}]){
   const r=await handle(new Request('https://api.test/api/v1/tools'),{...env,...overrides});expect(r.status).toBe(503);expect(await r.text()).not.toContain('SENTINEL');expect(r.headers.get('cache-control')).toBe('no-store');
  }
 });
 it('never returns a cached published detail after unpublishing, and detects inconsistent stored metadata',async()=>{
  const path='/tool-10000',original=await fetchApi(path),etag=original.headers.get('etag')!;
  expect(original.status).toBe(200);
  try {
   await env.DB.prepare("UPDATE tools SET status='archived' WHERE id='tool-10000'").run();
   const hidden=await fetchApi(path,{headers:{'If-None-Match':etag}});expect(hidden.status).toBe(404);expect(hidden.headers.get('cache-control')).toBe('no-store');
   await env.DB.prepare("UPDATE tools SET status='published',name='Inconsistent TEST metadata' WHERE id='tool-10000'").run();
   const corrupt=await fetchApi(path);expect(corrupt.status).toBe(503);expect((await corrupt.json() as {error:{code:string}}).error.code).toBe('INTEGRITY_ERROR');
   expect((await fetchApi('?q=tool%20000&limit=100')).status).toBe(503);
  }finally{await env.DB.prepare("UPDATE tools SET status='published',name='Tool 000' WHERE id='tool-10000'").run()}
 });
});
