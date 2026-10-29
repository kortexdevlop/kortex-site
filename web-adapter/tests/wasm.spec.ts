import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {serve,root} from '../../tests/web/harness';
import {prepare} from './helpers';
let server:Awaited<ReturnType<typeof serve>>;
test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>server.close());
test.beforeEach(async({page})=>{await page.goto(server.origin);await prepare(page)});
test('module signé réel en Worker : multiply, corpus, 100 actions et refus unsigned',async({page})=>{
 const cases=JSON.parse(readFileSync(resolve(root,'tests/spikes/native-runner/wire-cases.json'),'utf8'));
 const result=await page.evaluate(async cases=>{
  const t=(window as any).adapterTest;const bridge=await t.api.startVerifiedTool(t.tool);
  const req=(kind:string,payload={})=>({v:1,request_id:crypto.randomUUID(),kind,payload});
  const init=await bridge.dispatch(req('init'));const render=await bridge.dispatch(req('render'));const action=await bridge.dispatch(req('action.invoke',{action_id:'multiply',input:{a:25,b:48}}));
  for(let i=0;i<100;i++){const r=await bridge.dispatch(req('action.invoke',{action_id:'multiply',input:{a:i,b:2}}));if(r.data.result!==i*2)throw Error('PARITY');}
  const corpus=[];for(const item of cases){const value=item.request??(()=>{try{return JSON.parse(item.raw)}catch{return undefined}})();try{const r=await bridge.dispatch(value);corpus.push({name:item.name,ok:r.ok,code:r.error?.code,result:r.data?.result});}catch(e){corpus.push({name:item.name,rejected:true});}}
  bridge.stop();let unsigned=false;try{await t.api.startVerifiedTool({bytes:t.bytes,manifest:t.manifest})}catch{unsigned=true;}
  const leaked=t.tool.bytes;leaked.fill(0);const again=await t.api.startVerifiedTool(t.tool);again.stop();
  return {init,render,action,corpus,unsigned};
 },cases);
 expect(result.init.ok).toBe(true);expect(result.render.data.type).toBe('text');expect(result.action.data.result).toBe(1200);expect(result.unsigned).toBe(true);
 for(let i=0;i<cases.length;i++){const actual=result.corpus[i],expected=cases[i];if(actual.rejected){expect(expected.ok===false||expected.request?.kind==='capability.result').toBe(true);}else{expect(actual.ok,expected.name).toBe(expected.ok);if(expected.code)expect(actual.code).toBe(expected.code);if(expected.result!==undefined)expect(actual.result).toBe(expected.result);}}
});
test('buffers hors limites, module invalide et métadonnées altérées refusés',async({page})=>{
 const result=await page.evaluate(async()=>{
  const t=(window as any).adapterTest, memory=await import('/adapter/memory.js' as string);let rejected=0;
  const m=new WebAssembly.Memory({initial:1,maximum:1024});for(const [p,n] of [[65530,20],[-1,1],[0,0],[0,1048577],[0xffffffff,2]])try{memory.range(m,p,n)}catch{rejected++;}
  try{await t.admit(new Uint8Array([1,2,3]))}catch{rejected++;}
  try{await t.admit(t.bytes,{...t.manifest,permissions:['network.fetch']})}catch{rejected++;}
  return rejected;
 });expect(result).toBe(7);
});
test('le shell Dioxus reçoit le vrai rendu du Worker',async({page})=>{
 await page.evaluate(async()=>{const t=(window as any).adapterTest;await (window as any).kortexHost.startVerifiedTool(t.tool,{api_origin:location.origin,allowed_parent_origin:location.origin,tool_id:t.manifest.id,tool_version:t.manifest.version});});
 await expect(page.locator('main')).toHaveAttribute('data-status','ready');await expect(page.locator('[data-node=result]')).toHaveText('Ready');
 await page.evaluate(async()=>{await (window as any).kortexHost.command({kind:'action.invoke',payload:{action_id:'multiply',input:{a:25,b:48}}})});
 await expect(page.getByRole('status').first()).toBeVisible();await expect(page.getByLabel('Résultat')).toContainText('1200');
});

test('worker interrompu sur boucle CPU, buffers invalides et mauvaise corrélation',async({page})=>{
 const result=await page.evaluate(async()=>{
  const t=(window as any).adapterTest;
  const u=(n:number)=>{const b=[];do{let v=n&127;n=Math.floor(n/128);if(n)v|=128;b.push(v)}while(n);return b};
  const s=(value:bigint)=>{const b=[];let n=value;for(;;){let v=Number(n&127n);n>>=7n;const done=(n===0n&&!(v&64))||(n===-1n&&!!(v&64));if(!done)v|=128;b.push(v);if(done)return b}};
  const sec=(id:number,b:number[])=>[id,...u(b.length),...b];const text=(v:string)=>{const b=[...new TextEncoder().encode(v)];return [...u(b.length),...b]};
  async function module(mode:string){
   const reply=new TextEncoder().encode(JSON.stringify({v:1,request_id:'00000000-0000-4000-8000-000000000000',ok:true,data:{}}));
   const dispatch=mode==='loop'?[0,3,64,12,0,11,66,0,11]:[0,66,...s(mode==='buffer'?(65530n<<32n|100n):(1024n<<32n|BigInt(reply.length))),11];
   const code=[[0,65,...s(64n),11],[0,11],dispatch].flatMap(body=>[...u(body.length),...body]);
   const bytes=new Uint8Array([0,97,115,109,1,0,0,0,...sec(1,[3,96,1,127,1,127,96,2,127,127,0,96,2,127,127,1,126]),...sec(3,[3,0,1,2]),...sec(5,[1,1,1,...u(1024)]),...sec(7,[4,...text('memory'),2,0,...text('kortex_alloc'),0,0,...text('kortex_dealloc'),0,1,...text('kortex_dispatch'),0,2]),...sec(10,[3,...code]),...sec(11,[1,0,65,...s(1024n),11,...u(reply.length),...reply]),...t.section('kortex.manifest',t.manifest),...t.section('kortex.contract',t.contract)]);
   return t.admit(bytes);
  }
  const errors=[];let ticks=0;const timer=setInterval(()=>ticks++,10);
  try{for(const mode of ['loop','buffer','correlation']){const tool=await module(mode);const bridge=await t.api.startVerifiedTool(tool,mode==='loop'?250:30000);try{await bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind:'init',payload:{}});errors.push('ACCEPT')}catch(e){errors.push((e as Error).message)}finally{bridge.stop()}}}finally{clearInterval(timer)}
  return {errors,ticks};
 });expect(result.errors).toEqual(['TIMEOUT','INVALID_BUFFER','CORRELATION_ERROR']);expect(result.ticks).toBeGreaterThan(0);
});

test('calculatrice réelle : archive signée et goldens SDK inchangés dans le Worker',async({page})=>{
 const f=JSON.parse(readFileSync(resolve(root,'target/calculatrice-fixture/fixture.json'),'utf8'));
 const reference=JSON.parse(readFileSync(resolve(root,'tests/golden/calculatrice-reference.json'),'utf8'));expect(f.guest_sha256).toBe(reference.guest_sha256);expect(f.descriptor.sha256).toBe(reference.archive_sha256);
 const cases=readFileSync(resolve(root,'tests/golden/calculatrice.ndjson'),'utf8').trim().split('\n').map(line=>JSON.parse(line));
 const actual=await page.evaluate(async({f,cases})=>{
  const api=await import('/adapter/interop.js' as string);
  const tool=await new api.ToolLoader(new api.TrustStore(f.keys),location.origin).loadTool({...f.descriptor,url:location.origin+'/fixtures/calculatrice.tool'});
  const bridge=await api.startVerifiedTool(tool);const replies=[];
  try{for(const c of cases)replies.push(await bridge.dispatch(c.request));}finally{bridge.stop()}
  return replies;
 },{f,cases});expect(actual).toEqual(cases.map(c=>c.reply));
});
test('fresh compiler output executes the same frozen business goldens across checkout paths',async({page})=>{
 const f=JSON.parse(readFileSync(resolve(root,'target/calculatrice-current-fixture/fixture.json'),'utf8'));
 const cases=readFileSync(resolve(root,'tests/golden/calculatrice.ndjson'),'utf8').trim().split('\n').map(line=>JSON.parse(line));
 await page.goto(server.origin);
 const actual=await page.evaluate(async({f,cases})=>{const api=await import('/adapter/interop.js' as string);const tool=await new api.ToolLoader(new api.TrustStore(f.keys),location.origin).loadTool({...f.descriptor,url:location.origin+'/fixtures/calculatrice-current.tool'});const bridge=await api.startVerifiedTool(tool),replies=[];try{for(const c of cases)replies.push(await bridge.dispatch(c.request));return replies}finally{bridge.stop()}},{f,cases});expect(actual).toEqual(cases.map(c=>c.reply));
});
