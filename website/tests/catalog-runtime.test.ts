import {test} from 'node:test';
import assert from 'node:assert/strict';
import {catalogQuery,nextPage} from '../src/lib/catalog.ts';
import {runtimeOrigin,connectRuntime} from '../src/lib/runtime-bridge.ts';
import {appUrl} from '../src/lib/urls.ts';
test('catalogue query is bounded, cursor retains its filter scope and URLs are encoded',()=>{
 const q=catalogQuery(new URLSearchParams({q:'x'.repeat(201),category:'image',sort:'name',cursor:'a+b/='}));assert.equal(q.get('limit'),'24');assert.equal(q.get('q')!.length,100);assert.equal(new URL(nextPage(q,'new+cursor')!,'https://kortex.site').searchParams.get('cursor'),'new+cursor');assert.equal(nextPage(q,null),null);
 assert.equal(catalogQuery(new URLSearchParams('category=evil%2Fid&sort=evil')).get('category'),null);assert.equal(appUrl('calculatrice'),'/app/calculatrice');assert.throws(()=>appUrl('../admin'));
});
test('runtime deployment allowlist refuses same origin, arbitrary HTTPS and nonlocal HTTP',()=>{
 for(const value of ['https://evil.invalid','https://run.kortex.site/path','https://kortex.site','http://run.kortex.site','https://run.kortex.site@evil.invalid','http://127.0.0.1:4340'])assert.equal(runtimeOrigin(value,'https://kortex.site'),null);
 assert.equal(runtimeOrigin('https://run.kortex.site','https://kortex.site'),'https://run.kortex.site');assert.equal(runtimeOrigin('http://127.0.0.1:4340','http://127.0.0.1:4321'),'http://127.0.0.1:4340');
});
test('bridge validates source, origin, nonce, session, tool/version and disposes listeners',()=>{
 const originalWindow=globalThis.window,originalLocation=globalThis.location;const events=new EventTarget(),child={postMessage:(message:unknown)=>messages.push(message)},messages:any[]=[],states:string[]=[];
 Object.assign(globalThis,{window:events,location:{origin:'http://127.0.0.1:4321'}});
 const frame=Object.assign(new EventTarget(),{contentWindow:child}) as unknown as HTMLIFrameElement;
 try{const stop=connectRuntime(frame,'http://127.0.0.1:4340','calculatrice','1.0.0',s=>states.push(s));const nonce=messages[0].nonce;const session='00000000-0000-4000-8000-000000000000';
 const send=(data:unknown,origin='http://127.0.0.1:4340',source:any=child)=>events.dispatchEvent(Object.assign(new Event('message'),{data,origin,source}));
 const ready={kind:'READY',nonce,session_id:session};send(ready,'https://evil.invalid');send(ready,undefined,{});send({...ready,nonce:'b'.repeat(64)});assert.equal(messages.length,1);send(ready);assert.equal(messages[1].kind,'OPEN');
 const result={kind:'RESULT',nonce,session_id:session,result:{tool_id:'calculatrice',version:'1.0.0',status:'ready'}};
 send({...result,session_id:'wrong'});send({...result,result:{...result.result,version:'2.0.0'}});assert.equal(states.at(-1),'loading');send(result);assert.equal(states.at(-1),'ready');stop();send(result);assert.equal(states.filter(s=>s==='ready').length,1);
 }finally{Object.assign(globalThis,{window:originalWindow,location:originalLocation});}
});
