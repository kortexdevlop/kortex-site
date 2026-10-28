import {test,expect} from 'playwright/test';
import {createServer} from 'node:http';
import {readFileSync} from 'node:fs';
import {resolve,extname,sep} from 'node:path';
import {randomBytes} from 'node:crypto';
import {root} from './harness';
const {makeTool}=require('../fixtures/make-tool.cjs');
test('iframe Dioxus : paquet signé, CSP stricte, isolation, nonce et recovery',async({page,browser})=>{
 const fixture=makeTool();let parentOrigin='',hostOrigin='';const dist=resolve(root,'target/web-dist');
 const host=createServer((req,res)=>{try{const nonce=randomBytes(18).toString('base64');res.setHeader('Content-Security-Policy',`default-src 'none'; script-src 'self' 'nonce-${nonce}' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'; img-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors ${parentOrigin}`);res.setHeader('Referrer-Policy','no-referrer');res.setHeader('X-Content-Type-Options','nosniff');const path=new URL(req.url!,'http://local').pathname;
 if(path==='/embed-config.js'){res.setHeader('Content-Type','text/javascript');res.end(`window.kortexEmbedConfig=${JSON.stringify({parent_origin:parentOrigin,api_origin:hostOrigin,keys:fixture.keys,artifacts:[{...fixture.descriptor,url:hostOrigin+'/demo.tool'}]})};`);return}
 if(path==='/demo.tool'){res.setHeader('Content-Type','application/zip');res.end(fixture.bytes);return}
 if(path==='/frame.html'){res.setHeader('Content-Type','text/html');let html=readFileSync(resolve(root,'runtime/web/public/frame.html'),'utf8');html=html.replace(/<script\b/g,`<script nonce="${nonce}"`);res.end(html);return}
 const file=resolve(dist,'.'+path);if(!file.startsWith(dist+sep))throw Error('path');res.setHeader('Content-Type',({'.js':'text/javascript','.wasm':'application/wasm','.css':'text/css'} as any)[extname(file)]||'application/octet-stream');res.end(readFileSync(file));
 }catch{res.statusCode=404;res.end()}});
 await new Promise<void>(r=>host.listen(0,'127.0.0.1',r));hostOrigin=`http://127.0.0.1:${(host.address() as any).port}`;
 const parent=createServer((req,res)=>{res.setHeader('Content-Type','text/html; charset=utf-8');res.end(`<!doctype html><html><body><h1>Catalogue de test</h1><button id="open">Ouvrir l’outil</button><output id="result"></output><iframe sandbox="allow-scripts allow-same-origin" src="${hostOrigin}/frame.html"></iframe><script>
 const frame=document.querySelector('iframe');let nonce;
 document.querySelector('#open').onclick=()=>{nonce=Array.from(crypto.getRandomValues(new Uint8Array(32)),b=>b.toString(16).padStart(2,'0')).join('');frame.contentWindow.postMessage({kind:'HELLO',nonce,client_version:'1.0'},'${hostOrigin}')};
 addEventListener('message',e=>{if(e.source!==frame.contentWindow||e.origin!=='${hostOrigin}'||e.data.nonce!==nonce)return;if(e.data.kind==='READY')frame.contentWindow.postMessage({kind:'OPEN',nonce,session_id:e.data.session_id,tool_id:'${fixture.descriptor.tool_id}',version:'1.0.0'},'${hostOrigin}');if(e.data.kind==='RESULT')document.querySelector('#result').textContent=e.data.result.status});
 </script></body></html>`)});await new Promise<void>(r=>parent.listen(0,'127.0.0.1',r));parentOrigin=`http://127.0.0.1:${(parent.address() as any).port}`;
 try{
  await page.goto(parentOrigin);const frame=page.frameLocator('iframe');await expect(frame.getByRole('heading',{name:'Kortex',exact:true})).toBeVisible();
  await page.evaluate(()=>{(window as any).messages=[];window.addEventListener('message',e=>(window as any).messages.push(e.data))});
  const send=(data:any)=>page.evaluate(({origin,data})=>document.querySelector('iframe')!.contentWindow!.postMessage(data,origin),{origin:hostOrigin,data});
  await send({kind:'HELLO',nonce:'b'.repeat(64),client_version:'wrong'});await page.waitForTimeout(100);expect(await page.evaluate(()=>(window as any).messages.length)).toBe(0);
  await send({kind:'HELLO',nonce:'a'.repeat(64),client_version:'1.0'});await expect.poll(()=>page.evaluate(()=>(window as any).messages.filter((v:any)=>v.kind==='READY').length)).toBe(1);await send({kind:'HELLO',nonce:'a'.repeat(64),client_version:'1.0'});await page.waitForTimeout(100);expect(await page.evaluate(()=>(window as any).messages.filter((v:any)=>v.kind==='READY').length)).toBe(1);
  const ready=await page.evaluate(()=>(window as any).messages.find((v:any)=>v.kind==='READY'));
  await send({kind:'OPEN',nonce:'a'.repeat(64),session_id:ready.session_id,tool_id:fixture.descriptor.tool_id,version:'1.0.0'});await expect(frame.locator('[data-node=result]')).toHaveText('Ready');
  expect(await page.evaluate(()=>{try{void document.querySelector('iframe')!.contentWindow!.localStorage;return false}catch{return true}})).toBe(true);
  expect(await page.evaluate(()=>{try{void (document.querySelector('iframe')!.contentWindow as any).kortexHost;return false}catch{return true}})).toBe(true);
  const child=page.frames().find(f=>f.url().startsWith(hostOrigin))!;
  const invocation={kind:'WORKFLOW_INVOKE',nonce:'a'.repeat(64),session_id:ready.session_id,request_id:'16300000-0000-4000-8000-000000000001',action_id:'multiply',input:{a:25,b:48}};
  await send({...invocation,nonce:'b'.repeat(64)});await send({...invocation,session_id:'wrong-session'});
  await send(invocation);await expect.poll(()=>page.evaluate(()=>(window as any).messages.filter((v:any)=>v.kind==='AI_RESULT').length)).toBe(1);
  const local=await page.evaluate(()=>(window as any).messages.find((v:any)=>v.kind==='AI_RESULT').result);expect(local.data.result).toBe(1200);expect(local.executed_locally).toBe(true);expect(local.shareable).toBe(false);
  await send(invocation);await page.waitForTimeout(100);expect(await page.evaluate(()=>(window as any).messages.filter((v:any)=>v.kind==='AI_RESULT').length)).toBe(1);await send({...invocation,request_id:'16300000-0000-4000-8000-000000000002'});await expect.poll(()=>page.evaluate(()=>(window as any).messages.filter((v:any)=>v.kind==='AI_RESULT').length)).toBe(2);
  const blocked=await child.evaluate(async()=>{let violation=false;document.addEventListener('securitypolicyviolation',()=>violation=true,{once:true});const script=document.createElement('script');script.textContent='window.__injected=true';document.body.append(script);await new Promise(r=>setTimeout(r,50));return violation&&!(window as any).__injected});expect(blocked).toBe(true);
  await child.evaluate(()=>{(window as any).kortexHost.bridge.stop('GUEST_CRASH')});await expect(frame.locator('main')).toHaveAttribute('data-status','error');await send({kind:'OPEN',nonce:'a'.repeat(64),session_id:ready.session_id,tool_id:fixture.descriptor.tool_id,version:'1.0.0'});await page.waitForTimeout(100);await expect(frame.locator('main')).toHaveAttribute('data-status','error');
  const attacker=await browser.newPage();await attacker.goto('about:blank');await attacker.setContent(`<iframe src="${hostOrigin}/frame.html"></iframe>`);await attacker.waitForTimeout(200);expect(await attacker.locator('iframe').contentFrame().getByRole('heading',{name:'Kortex',exact:true}).count()).toBe(0);await attacker.close();
  await page.reload();await expect(frame.getByRole('heading',{name:'Kortex',exact:true})).toBeVisible();await page.getByRole('button',{name:'Ouvrir l’outil'}).click();await expect(page.locator('#result')).toHaveText('ready');
 }finally{parent.close();host.close();parent.closeAllConnections();host.closeAllConnections();}
});
