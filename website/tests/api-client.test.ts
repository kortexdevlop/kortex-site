import {test} from 'node:test';
import assert from 'node:assert/strict';
import {ApiClient,ApiUnavailable} from '../src/lib/api-client.ts';
import {proxy} from '../src/lib/proxy.ts';
import {canonical} from '../src/lib/urls.ts';
import {isToolSummary,palette} from '../src/lib/tools.ts';
import fixtures from '../../tests/visual/design-fixtures.json' with {type:'json'};
import manifest from '../../tools/calculatrice/manifest.json' with {type:'json'};
import contract from '../../tools/calculatrice/contract.json' with {type:'json'};
import descriptor from '../../tests/fixtures/artifact-signed-test.json' with {type:'json'};
test('real schemas validate demo summaries, stable category colors and reject hostile metadata',()=>{
  for(const tool of fixtures.tools){assert.ok(isToolSummary(tool));assert.equal(palette(tool as any),palette({...tool,id:'different-id'} as any));}
  assert.equal(isToolSummary({...fixtures.tools[0],icon_path:'https://attacker.invalid/icon.svg'}),false);
  assert.equal(isToolSummary({...fixtures.tools[0],unexpected:true}),false);
});
test('SSR client sends only an absolute binding request and validates bounded API responses',async()=>{
  let sent:Request|undefined;
  const client=new ApiClient({async fetch(request){sent=request;return Response.json({items:[fixtures.tools[0]],next_cursor:null});}});
  const list=await client.listTools(new URLSearchParams({q:'texte & mesure'}));assert.equal(list.items[0].id,fixtures.tools[0].id);
  assert.equal(sent?.url,'https://kortex-api.internal/api/v1/tools?q=texte+%26+mesure');
  assert.equal(sent?.headers.has('authorization'),false);
  await assert.rejects(new ApiClient().listTools(),ApiUnavailable);
  for(const response of [Response.json({items:['bad']}),new Response('bad',{status:503}),new Response('x'.repeat(1048577),{headers:{'Content-Type':'application/json'}})])await assert.rejects(new ApiClient({async fetch(){return response;}}).listTools(),ApiUnavailable);
  await assert.rejects(new ApiClient({async fetch(){return new Response(null,{status:404});}}).getTool('absent'),(e:unknown)=>e instanceof ApiUnavailable&&e.status===404);
});
test('same-origin read proxy strips spoofable auth headers, preserves cookies privately and streams bytes',async()=>{
  let sent:Request|undefined;
  const binding={async fetch(request:Request){sent=request;return new Response('bytes',{headers:{'Content-Type':'application/octet-stream','Set-Cookie':'session=opaque; Path=/; HttpOnly; Secure; SameSite=Lax','Cache-Control':'public, max-age=3600','Access-Control-Allow-Origin':'*'}});}};
  const response=await proxy(new Request('https://kortex.site/api/v1/tools',{headers:{Origin:'https://kortex.site',Cookie:'session=opaque',Authorization:'Bearer attacker','X-Kortex-Admin':'true','X-Forwarded-Host':'attacker.invalid'}}),binding,'https://kortex.site');
  assert.equal(response.status,200);assert.equal(await response.text(),'bytes');
  assert.equal(sent?.headers.get('cookie'),'session=opaque');assert.equal(sent?.headers.has('authorization'),false);assert.equal(sent?.headers.has('x-kortex-admin'),false);assert.equal(sent?.headers.has('x-forwarded-host'),false);
  assert.equal(response.headers.get('cache-control'),'private, no-store');assert.equal(response.headers.has('access-control-allow-origin'),false);assert.match(response.headers.get('set-cookie')!,/HttpOnly/);
});
test('detail and actions clients accept schema-valid API envelopes',async()=>{
  const tool={id:manifest.id,version:manifest.version,name:manifest.name,summary:manifest.summary,category:manifest.category,tags:manifest.tags,icon_path:manifest.icon_path,abi_version:manifest.abi_version,sha256:descriptor.sha256,published_at:1};
  const client=new ApiClient({async fetch(request){return Response.json(request.url.endsWith('/actions')?{tool_id:manifest.id,version:manifest.version,actions:contract.actions}:{tool,manifest,contract,descriptor});}});
  assert.equal((await client.getTool(manifest.id)).tool.id,manifest.id);
  assert.equal((await client.getActions(manifest.id)).tool_id,manifest.id);
});
test('a binding that never responds cannot hold SSR or the proxy open indefinitely',async()=>{
  const hung={fetch:()=>new Promise<Response>(()=>{})};
  const results=await Promise.allSettled([new ApiClient(hung).listTools(),proxy(new Request('https://kortex.site/api/v1/tools'),hung,'https://kortex.site')]);
  assert.equal(results[0].status,'rejected');assert.ok(results[1].status==='fulfilled'&&results[1].value.status===503);
});
test('proxy rejects foreign origins, writes, admin paths, redirects and unsafe session cookies',async()=>{
  let calls=0;const binding={async fetch(){calls++;return Response.json({});}};
  for(const [url,options,status] of [
    ['https://kortex.site/api/v1/tools',{headers:{Origin:'https://evil.invalid'}},403],
    ['https://evil.invalid/api/v1/tools',{},403],
    ['https://kortex.site/api/v1/tools',{headers:{'Sec-Fetch-Site':'cross-site'}},403],
    ['https://kortex.site/api/v1/tools',{method:'POST',body:'{}'},405],
    ['https://kortex.site/api/v1/admin/publish',{},404],
    ['https://kortex.site/api/v1/tools/%2e%2e/admin',{},404],
  ] as const){assert.equal((await proxy(new Request(url,options),binding,'https://kortex.site')).status,status);}
  assert.equal(calls,0);
  for(const upstream of [new Response(null,{status:302,headers:{Location:'https://evil.invalid'}}),new Response('bad',{headers:{'Set-Cookie':'session=unsafe'}})])assert.equal((await proxy(new Request('https://kortex.site/api/v1/tools'),{async fetch(){return upstream;}},'https://kortex.site')).status,503);
  assert.equal((await proxy(new Request('https://kortex.site/api/v1/tools'),undefined,'https://kortex.site')).status,503);
});
test('canonical URLs remain on the configured HTTPS origin',()=>{
  assert.equal(canonical('/tools?x=1#test','https://kortex.site'),'https://kortex.site/tools?x=1');
  for(const path of ['//evil.invalid','/\\evil.invalid','https://evil.invalid'])assert.throws(()=>canonical(path,'https://kortex.site'));
  assert.throws(()=>canonical('/','http://evil.invalid'));
});
test('proxy accepts numeric-leading IDs and encoded SemVer build metadata without losing integrity headers',async()=>{
  const paths:string[]=[];const digest='a'.repeat(64);
  const binding={async fetch(request:Request){paths.push(new URL(request.url).pathname);return new Response('exact bytes',{headers:{'X-Kortex-SHA256':digest}});}};
  for(const path of ['/api/v1/tools/123-tool','/api/v1/tools/123-tool/actions','/api/v1/downloads/123-tool/1.0.0%2Bbuild.1']){
    const response=await proxy(new Request('https://kortex.site'+path),binding,'https://kortex.site');assert.equal(response.status,200);assert.equal(response.headers.get('x-kortex-sha256'),digest);assert.equal(await response.text(),'exact bytes');
  }
  assert.equal(paths.length,3);assert.equal(paths[2],'/api/v1/downloads/123-tool/1.0.0%2Bbuild.1');
  assert.equal((await proxy(new Request('https://kortex.site/api/v1/downloads/123-tool/1.0.0%2Fadmin'),binding,'https://kortex.site')).status,404);
});
