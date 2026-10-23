import { test, expect, chromium, firefox, webkit } from 'playwright/test';
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { resolve, extname, sep } from 'node:path';

const root=resolve(__dirname,'../../..');
let server;let origin;
test.beforeAll(async()=>{
 server=createServer((req,res)=>{try{const path=resolve(root,'.'+new URL(req.url,'http://local').pathname);if(!path.startsWith(root+sep))throw Error();const mime={'.html':'text/html','.ts':'text/javascript','.wasm':'application/wasm'};res.setHeader('Content-Type',mime[extname(path)]||'application/octet-stream');res.end(readFileSync(path));}catch{res.statusCode=404;res.end();}});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));origin=`http://127.0.0.1:${server.address().port}`;
});
test.afterAll(async()=>{await new Promise(resolve=>server.close(resolve));});
for(const [name,type] of Object.entries({chromium,firefox,webkit}))test(`${name}: real Core WASM ABI in worker`,async()=>{
 test.setTimeout(120000);const browser=await type.launch({headless:true});
 try{const page=await browser.newPage();await page.goto(origin+'/tests/spikes/browser-runner/index.html');
 const call=payload=>page.evaluate(p=>window.call(p),payload);
 const id='00000000-0000-4000-8000-000000000001';
 const req=(kind,payload={})=>({v:1,request_id:id,kind,payload});
 const info=await call({op:'load'});expect(info.imports).toEqual([{module:'kortex_v1',name:'host_enqueue',kind:'function'}]);
 expect(info.exports.map(x=>x.name)).toEqual(expect.arrayContaining(['memory','kortex_alloc','kortex_dealloc','kortex_dispatch']));
 const init=await call({request:req('init')});expect(init.ok).toBe(true);expect(init.data.enqueue_status).toBe(-1);
 const render=await call({request:req('render')});expect(render.data.type).toBe('text');
 const multiply=req('action.invoke',{action_id:'multiply',input:{a:25,b:48}});
 const result=await call({request:multiply});expect(result.data.result).toBe(1200);expect(result.request_id).toBe(id);
 const repeated=await call({op:'repeat',count:10000,request:multiply});expect(repeated.count).toBe(10000);expect(repeated.allocations).toBe(0);expect(repeated.memory).toBeLessThanOrEqual(67108864);console.log(name,JSON.stringify(repeated));
 expect((await call({raw:'{'})).error.code).toBe('INVALID_JSON');
 const large=await page.evaluate(async()=>{try{await window.call({raw:' '.repeat(1048577)});return 'accepted'}catch(e){return e.message}});expect(large).toContain('1 MiB');
 expect((await call({op:'oversized-guest'})).error.code).toBe('INVALID_BUFFER');
 await page.evaluate(()=>window.restart());await call({op:'load'});expect((await call({request:req('init')})).ok).toBe(true);expect((await call({request:multiply})).data.result).toBe(1200);
 }finally{await browser.close();}
});
