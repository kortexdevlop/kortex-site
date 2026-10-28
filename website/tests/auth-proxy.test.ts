import test from 'node:test';
import assert from 'node:assert/strict';
import {proxy} from '../src/lib/proxy.ts';
const web='https://kortex.site';
test('password form JSON is bounded and forwarded privately with CSRF context',async()=>{
 let sent:Request|undefined;const binding={async fetch(r:Request){sent=r;assert.deepEqual(await r.json(),{email:'test@example.test',password:'synthetic-test-phrase'});return Response.json({ok:true})}};
 const response=await proxy(new Request(web+'/api/v1/auth/password/login',{method:'POST',headers:{Origin:web,'Content-Type':'application/json','X-CSRF-Token':'proof'},body:JSON.stringify({email:'test@example.test',password:'synthetic-test-phrase'})}),undefined,web,binding);assert.equal(response.status,200);assert.equal(sent?.headers.get('Content-Type'),'application/json');assert.equal(sent?.headers.get('X-CSRF-Token'),'proof');assert.equal(new URL(sent!.url).search,'');
 assert.equal((await proxy(new Request(web+'/api/v1/auth/password/login',{method:'POST',headers:{Origin:web},body:'x'.repeat(4097)}),undefined,web,binding)).status,403);
 assert.equal((await proxy(new Request(web+'/api/v1/auth/password/login'),undefined,web,binding)).headers.get('Allow'),'POST');
});
test('auth uses a dedicated binding and strips bearer, forwarded-host and operator headers',async()=>{
 let sent:Request|undefined;const binding={async fetch(r:Request){sent=r;return Response.json({user:null})}};
 assert.equal((await proxy(new Request(web+'/api/v1/me'),binding,web)).status,503);
 const response=await proxy(new Request(web+'/api/v1/me',{headers:{Cookie:'__Host-kortex_session=opaque','X-Forwarded-Host':'evil','X-Kortex-Auth-Service':'true'}}),undefined,web,binding);
 assert.equal(response.status,200);assert.equal(sent?.url,'https://kortex-api.internal/api/v1/me');assert.equal(sent?.headers.has('X-Forwarded-Host'),false);assert.equal(sent?.headers.has('X-Kortex-Auth-Service'),false);assert.equal(response.headers.get('Cache-Control'),'private, no-store');assert.equal(response.headers.has('Access-Control-Allow-Origin'),false);
 assert.equal((await proxy(new Request(web+'/api/v1/me',{headers:{Authorization:'Bearer injected'}}),undefined,web,binding)).status,403);
});
test('auth forwards only safe logout context and refuses another origin or cross-site initiation',async()=>{
 let sent:Request|undefined;const binding={async fetch(r:Request){sent=r;return new Response(null,{status:204})}};
 const response=await proxy(new Request(web+'/api/v1/auth/logout',{method:'POST',headers:{Origin:web,Cookie:'opaque','X-CSRF-Token':'test'}}),undefined,web,binding);assert.equal(response.status,204);assert.equal(sent?.headers.get('Origin'),web);assert.equal(sent?.headers.get('X-CSRF-Token'),'test');
 for(const path of ['login','logout'])assert.equal((await proxy(new Request(web+'/api/v1/auth/'+path,{method:path==='logout'?'POST':'GET',headers:{'Sec-Fetch-Site':'cross-site'}}),undefined,web,binding)).status,403);
 assert.equal((await proxy(new Request(web+'/api/v1/me',{headers:{Origin:'https://run.kortex.site'}}),undefined,web,binding)).status,403);
});
test('Google navigation and OAuth callback redirects are narrowly allowed with no-referrer',async()=>{
 const login={async fetch(){return new Response(null,{status:302,headers:{Location:'https://accounts.google.com/o/oauth2/v2/auth?state=opaque'}})}};
 const response=await proxy(new Request(web+'/api/v1/auth/login'),undefined,web,login);assert.equal(response.status,302);assert.equal(response.headers.get('Referrer-Policy'),'no-referrer');
 const callback={async fetch(){return new Response(null,{status:303,headers:{Location:web+'/app'}})}};assert.equal((await proxy(new Request(web+'/api/v1/auth/callback?code=test&state=test',{headers:{'Sec-Fetch-Site':'cross-site'}}),undefined,web,callback)).status,303);
 for(const location of ['https://evil.test/','https://kortex.site/unexpected','https://kortex.site/app?token=forbidden'])assert.equal((await proxy(new Request(web+'/api/v1/auth/callback'),undefined,web,{async fetch(){return new Response(null,{status:303,headers:{Location:location}})}})).status,503);
 assert.equal((await proxy(new Request(web+'/api/v1/auth/login'),undefined,web,{async fetch(){return new Response(null,{status:302,headers:{Location:'https://evil.test/'}})}})).status,503);
});
test('auth cookie forwarding requires host-only Kortex names, Secure, HttpOnly, exact Path and SameSite',async()=>{
 const valid='__Host-kortex_session=opaque; Path=/; Secure; HttpOnly; SameSite=Lax';
 for(const value of [valid.replace('; Secure',''),valid.replace('; HttpOnly',''),valid+'; Domain=kortex.site',valid.replace('Path=/;','Path=/api;'),valid.replace('SameSite=Lax','SameSite=None'),valid.replace('__Host-kortex_session','unrelated')]){
  assert.equal((await proxy(new Request(web+'/api/v1/me'),undefined,web,{async fetch(){return new Response('{}',{headers:{'Set-Cookie':value}})}})).status,503);
 }
 const response=await proxy(new Request(web+'/api/v1/me'),undefined,web,{async fetch(){const headers=new Headers();headers.append('Set-Cookie',valid);headers.append('Set-Cookie','__Host-kortex_oidc=; Path=/; Max-Age=0; Secure; HttpOnly; SameSite=Lax');return new Response('{}',{headers})}});assert.equal(response.headers.getSetCookie().length,2);
});
