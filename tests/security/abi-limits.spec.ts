import {test,expect} from 'playwright/test';
import {serve} from '../web/harness';
import {prepare} from '../../web-adapter/tests/helpers';
import {writeFileSync} from 'node:fs';
let server:Awaited<ReturnType<typeof serve>>;test.beforeAll(async()=>server=await serve());test.afterAll(async()=>server.close());
test('signed hostile guests: alloc/dispatch/dealloc timeouts, invalid buffers, table and memory limits',async({page})=>{
 await page.goto(server.origin);await prepare(page);
 const result=await page.evaluate(async()=>{
  const t=(window as any).adapterTest;
  const u=(n:number)=>{const b=[];do{let v=n&127;n=Math.floor(n/128);if(n)v|=128;b.push(v)}while(n);return b};
  const s=(value:bigint)=>{const b=[];let n=value;for(;;){let v=Number(n&127n);n>>=7n;const done=(n===0n&&!(v&64))||(n===-1n&&!!(v&64));if(!done)v|=128;b.push(v);if(done)return b}};
  const sec=(id:number,b:number[])=>[id,...u(b.length),...b],text=(v:string)=>{const b=[...new TextEncoder().encode(v)];return [...u(b.length),...b]};
  const outcomes=[];let ticks=0;const timer=setInterval(()=>ticks++,10);
  for(const mode of ['alloc-loop','dispatch-loop','dealloc-loop','overflow','overlap','correlation','table-unbounded','table-large','memory-large','zero-control']){
   const id=crypto.randomUUID(),reply=new TextEncoder().encode(JSON.stringify({v:1,request_id:mode==='correlation'?crypto.randomUUID():id,ok:true,data:{}}));
   const packed=mode==='overflow'?-1n:mode==='overlap'?(64n<<32n|20n):(1024n<<32n|BigInt(reply.length));
   const alloc=mode==='alloc-loop'?[0,3,64,12,0,11,65,64,11]:[0,65,...s(mode==='zero-control'?0n:64n),11];
   const free=mode==='dealloc-loop'?[0,3,64,12,0,11,11]:[0,11];
   const call=mode==='dispatch-loop'?[0,3,64,12,0,11,66,0,11]:[0,66,...s(packed),11];
   const table=mode==='table-unbounded'?sec(4,[1,112,0,1]):mode==='table-large'?sec(4,[1,112,1,1,...u(65537)]):[];
   const bytes=new Uint8Array([0,97,115,109,1,0,0,0,...sec(1,[3,96,1,127,1,127,96,2,127,127,0,96,2,127,127,1,126]),...sec(3,[3,0,1,2]),...table,...sec(5,[1,1,1,...u(mode==='memory-large'?1025:1024)]),...sec(7,[4,...text('memory'),2,0,...text('kortex_alloc'),0,0,...text('kortex_dealloc'),0,1,...text('kortex_dispatch'),0,2]),...sec(10,[3,...[alloc,free,call].flatMap(body=>[...u(body.length),...body])]),...sec(11,[1,0,65,...s(1024n),11,...u(reply.length),...reply]),...t.section('kortex.manifest',t.manifest),...t.section('kortex.contract',t.contract)]);
   let bridge;const start=performance.now();try{const tool=await t.admit(bytes);bridge=await t.api.startVerifiedTool(tool,300);await bridge.dispatch({v:1,request_id:id,kind:'init',payload:{}});outcomes.push({mode,error:'ACCEPT',ms:performance.now()-start})}catch(e){outcomes.push({mode,error:(e as Error).message,ms:performance.now()-start})}finally{bridge?.stop()}
  }clearInterval(timer);
  const memory=await import('/adapter/memory.js' as string),m=new WebAssembly.Memory({initial:1,maximum:1024});let seed=0x18011802,mutations=0;for(let i=0;i<4096;i++){seed=(Math.imul(seed,1664525)+1013904223)>>>0;const ptr=seed;seed=(Math.imul(seed,1664525)+1013904223)>>>0;try{memory.range(m,ptr,seed);throw Error('ACCEPT')}catch(e){if((e as Error).message!=='INVALID_BUFFER')throw e;mutations++}}
  return{outcomes,ticks,mutations};
 });expect(result.outcomes.map(o=>o.error)).toEqual(['TIMEOUT','TIMEOUT','TIMEOUT','INVALID_BUFFER','OVERLAPPING_BUFFER','CORRELATION_ERROR','INVALID_WASM_ABI','INVALID_WASM_ABI','INVALID_WASM_ABI','ACCEPT']);expect(result.outcomes.every(o=>o.ms<5000)).toBe(true);expect(result.ticks).toBeGreaterThan(0);expect(result.mutations).toBe(4096);writeFileSync('docs/evidence/security-abi-web.json',JSON.stringify({date:new Date().toISOString(),passed:true,...result},null,2)+'\n');
});
