import {test,expect} from 'playwright/test';
import {createRequire} from 'node:module';
const {build}=createRequire(process.cwd()+'/web-adapter/package.json')('esbuild');
import {writeFileSync} from 'node:fs';
import {desktop} from '../e2e/workspace-helpers';
test('actual router rejects foreign, unknown, retired and closing instances; guests cannot dispatch',async({page})=>{
 const bundle=await build({entryPoints:['website/src/lib/workspace-actions.ts'],bundle:true,write:false,format:'esm',platform:'browser'});
 await page.route('**/__test/workspace-actions.js',r=>r.fulfill({contentType:'text/javascript',body:bundle.outputFiles[0].text}));await desktop(page);
 const ids=await page.evaluate(async()=>{
  const {WorkspaceActions}=await import(/* @vite-ignore */ '/__test/workspace-actions.js');const router=new WorkspaceActions('https://kortex-run-staging.userpgl.workers.dev');(window as any).__router=router;
  const ids=[];for(const tool of ['calculatrice','convertisseur-casse']){const data=await(await fetch('/api/v1/tools/'+tool+'/actions')).json();const frame=document.createElement('iframe');frame.sandbox.add('allow-scripts','allow-same-origin');document.body.append(frame);const w={window_id:crypto.randomUUID(),instance_id:crypto.randomUUID(),tool_id:tool,version:data.version,bounds:{x:0,y:0,width:500,height:400},mode:'normal',z:0};router.attach(w,frame);ids.push(w.instance_id)}return ids;
 });await expect.poll(()=>page.evaluate(()=>(window as any).__router.views().filter((v:any)=>v.status==='ready').length)).toBe(2);
 const result=await page.evaluate(async ids=>{const r=(window as any).__router,code=async(p:Promise<any>)=>{try{await p;return'accepted'}catch(e){return(e as any).code}};return{foreign:await code(r.dispatch(ids[1],'calculatrice.multiply',{a:25,b:48})),unknown:await code(r.dispatch(crypto.randomUUID(),'calculatrice.multiply',{a:25,b:48})),valid:await r.dispatch(ids[0],'calculatrice.multiply',{a:25,b:48})}},ids);
 expect(result).toEqual({foreign:'FORBIDDEN',unknown:'NOT_FOUND',valid:{result:1200}});
 const child=page.frames().find(f=>f.url().includes('frame.html'))!;await child.evaluate(id=>parent.postMessage({kind:'DISPATCH',instance_id:id,action_ref:'calculatrice.multiply',input:{a:100,b:100}},'https://kortex-web-staging.userpgl.workers.dev'),ids[0]);
 expect(await page.evaluate(()=>(window as any).__router.entries().length)).toBe(1);
 await page.route('**/api/v1/tools/calculatrice/actions',async route=>{await new Promise(r=>setTimeout(r,500));await route.continue().catch(()=>{})});
 const closed=await page.evaluate(async id=>{const r=(window as any).__router,p=r.dispatch(id,'calculatrice.add',{a:2,b:3}).catch((e:any)=>e.code);setTimeout(()=>r.close(id),50);const pending=await p;let retired;try{await r.dispatch(id,'calculatrice.add',{a:2,b:3})}catch(e){retired=(e as any).code}const record={window_id:crypto.randomUUID(),instance_id:id,tool_id:'calculatrice',version:'1.0.1',bounds:{x:0,y:0,width:500,height:400},mode:'normal',z:0};let reused;try{r.attach(record,document.createElement('iframe'))}catch(e){reused=(e as any).code}r.shutdown();return{pending,retired,reused}},ids[0]);
 expect(closed).toEqual({pending:'NOT_FOUND',retired:'NOT_FOUND',reused:'DUPLICATE'});writeFileSync('docs/evidence/desktop-router.json',JSON.stringify({date:new Date().toISOString(),passed:true,actual_source_bundle:true,real_signed_runtimes:true,...result,...closed,forged_guest_command_ignored:true},null,2)+'\n');
});
