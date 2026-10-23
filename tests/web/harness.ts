import {createServer} from 'node:http';
import {readFileSync} from 'node:fs';
import {resolve,extname,sep} from 'node:path';
export const root=resolve(__dirname,'../..');
export async function serve(){
 const server=createServer((req,res)=>{try{
  const pathname=new URL(req.url!,'http://localhost').pathname;
  const base=pathname.startsWith('/adapter/')?resolve(root,'web-adapter/dist'):resolve(root,'target/web-dist');
  let file=resolve(base,'.'+(pathname.startsWith('/adapter/')?pathname.slice(8):pathname));
  if(pathname==='/')file=resolve(base,'index.html');
  if(pathname==='/fixtures/guest.wasm')file=resolve(root,'tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm');
  else if(!file.startsWith(base+sep))throw Error('path');
  res.setHeader('Content-Type',({'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css','.json':'application/json'} as Record<string,string>)[extname(file)]||'application/octet-stream');
  res.end(readFileSync(file));
 }catch{res.statusCode=404;res.end('Not found');}});
 await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));
 return {origin:`http://127.0.0.1:${(server.address() as any).port}`,close:()=>new Promise<void>(done=>server.close(()=>done()))};
}
