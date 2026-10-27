import {createServer} from 'node:http';
import {readFileSync} from 'node:fs';
import {resolve,extname,sep} from 'node:path';
export const root=resolve(__dirname,'../..');
export async function serve(){
 const server=createServer((req,res)=>{try{
  const pathname=new URL(req.url!,'http://localhost').pathname;
  const base=resolve(root,'target/web-dist');
  let file=resolve(base,'.'+pathname);
  if(pathname==='/')file=resolve(base,'index.html');
  if(pathname==='/fixtures/calculatrice.tool')file=resolve(root,'target/calculatrice-fixture/calculatrice.tool');
  else if(pathname==='/fixtures/capability.wasm')file=resolve(root,'target/web-test-guest/wasm32-unknown-unknown/release/kortex_browser_capability_test.wasm');
  else if(pathname==='/fixtures/guest.wasm')file=resolve(root,'tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm');
  else if(!file.startsWith(base+sep))throw Error('path');
  res.setHeader('Content-Type',({'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css','.json':'application/json'} as Record<string,string>)[extname(file)]||'application/octet-stream');
  res.end(readFileSync(file));
 }catch{res.statusCode=404;res.end('Not found');}});
 await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));
 return {origin:`http://127.0.0.1:${(server.address() as any).port}`,close:()=>new Promise<void>(done=>server.close(()=>done()))};
}

// Ephemeral TLS test endpoint. The private key remains under ignored target/ only.
export async function serveNetwork(){
 const {createServer}=await import('node:https');const {existsSync,mkdirSync}=await import('node:fs');const {execFileSync}=await import('node:child_process');
 mkdirSync(resolve(root,'target'),{recursive:true});const key=resolve(root,'target/test-network.key'),cert=resolve(root,'target/test-network.crt');
 if(!existsSync(cert)){const openssl=process.platform==='win32'?'C:/Program Files/Git/usr/bin/openssl.exe':'openssl';execFileSync(openssl,['req','-x509','-newkey','rsa:2048','-nodes','-keyout',key,'-out',cert,'-days','1','-subj','/CN=api.example.test'],{stdio:'ignore'});}
 const server=createServer({key:readFileSync(key),cert:readFileSync(cert)},(req,res)=>{res.setHeader('Access-Control-Allow-Origin','*');if(req.url==='/redirect'){res.statusCode=302;res.setHeader('Location','https://evil.example.test/target');res.end();return}res.end(req.url==='/large'?'x'.repeat(256*1024+1):'hello');});
 await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));return {origin:`https://127.0.0.1:${(server.address() as any).port}`,close:()=>new Promise<void>(done=>server.close(()=>done()))};
}
