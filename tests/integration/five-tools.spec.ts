import {test,expect,type Page} from 'playwright/test';
import {createServer} from 'node:http';
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve,extname,sep} from 'node:path';
import {inflateRawSync} from 'node:zlib';
import {createHash} from 'node:crypto';
// Five real signed packages (never mocked) on the Web host; the same vectors drive the Windows Wasmtime test.
const root=process.cwd(),json=(p:string)=>JSON.parse(readFileSync(resolve(root,p),'utf8'));
const phase12=json('target/phase12/releases.json'),calculator=json('target/staging/calculatrice/release-report.json');
const releases=[{tool_id:'calculatrice',archive:calculator.archive,descriptor:calculator.descriptor,host_trust:calculator.host_trust,sha256:calculator.sha256},...phase12];
const vectors:{tool:string;action:string;input:unknown;output:unknown}[]=json('tests/integration/five-tools-vectors.json');
const sha=(bytes:Buffer)=>createHash('sha256').update(bytes).digest('hex');
// Minimal ZIP reader (central directory + raw inflate): enough to hash the guest module inside a .tool archive.
function zipEntry(zip:Buffer,name:string){let eocd=zip.length-22;while(eocd>=0&&zip.readUInt32LE(eocd)!==0x06054b50)eocd--;if(eocd<0)throw Error('ZIP_EOCD');let at=zip.readUInt32LE(eocd+16);
 for(let i=0,n=zip.readUInt16LE(eocd+10);i<n;i++){const method=zip.readUInt16LE(at+10),size=zip.readUInt32LE(at+20),nameLength=zip.readUInt16LE(at+28),extra=zip.readUInt16LE(at+30),comment=zip.readUInt16LE(at+32),local=zip.readUInt32LE(at+42),entry=zip.subarray(at+46,at+46+nameLength).toString('utf8');
  if(entry===name){const start=local+30+zip.readUInt16LE(local+26)+zip.readUInt16LE(local+28),data=zip.subarray(start,start+size);return method===0?data:inflateRawSync(data);}at+=46+nameLength+extra+comment;}
 throw Error('ZIP_ENTRY_MISSING '+name);}
function guestHash(archive:string){return sha(zipEntry(readFileSync(archive),'module.wasm'));}
let server:ReturnType<typeof createServer>,origin:string;const matrix:any[]=[];
test.beforeAll(async()=>{server=createServer((request,response)=>{try{const path=new URL(request.url!,'http://local').pathname,tool=releases.find(r=>path==='/tools/'+JSON.parse(readFileSync(r.descriptor,'utf8')).tool_id+'.tool');const base=resolve(root,'target/web-dist'),file=tool?tool.archive:path==='/'?resolve(base,'index.html'):resolve(base,'.'+path);if(!tool&&!file.startsWith(base+sep))throw Error('Path');response.setHeader('Content-Type',tool?'application/vnd.kortex.tool+zip':({'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css'} as any)[extname(file)]||'application/octet-stream');response.end(readFileSync(file));}catch{response.statusCode=404;response.end();}});await new Promise<void>(done=>server.listen(0,'127.0.0.1',done));origin='http://127.0.0.1:'+(server.address() as any).port;});
test.afterAll(async()=>{server.closeAllConnections();await new Promise<void>(done=>server.close(()=>done()));writeFileSync(resolve(root,'docs/evidence/five-tools-web.json'),JSON.stringify({date:new Date().toISOString(),host:'web-chromium-local-signed-packages',tools:matrix},null,2)+'\n');});
async function open(page:Page,release:any){const descriptor=JSON.parse(readFileSync(release.descriptor,'utf8')),keys=JSON.parse(readFileSync(release.host_trust,'utf8')).document.keys;await page.goto(origin);await page.evaluate(async({descriptor,keys,origin})=>{const api=await import('/adapter/interop.js' as string);await (window as any).kortexHost.loadTool({...descriptor,url:origin+'/tools/'+descriptor.tool_id+'.tool'},new api.TrustStore(keys),{api_origin:origin,allowed_parent_origin:origin,tool_id:descriptor.tool_id,tool_version:descriptor.version});},{descriptor,keys,origin});await expect(page.locator('main')).toHaveAttribute('data-status','ready');return descriptor;}
async function action(page:Page,id:string,input:unknown){await page.evaluate(async({id,input})=>{await (window as any).kortexHost.command({kind:'action.invoke',payload:{action_id:id,input}});},{id,input});return page.evaluate(()=>(window as any).kortexHost.snapshot().result);}
for(const release of releases){
 test(`${JSON.parse(readFileSync(release.descriptor,'utf8')).tool_id} : démarrage Web, actions identiques aux vecteurs et permissions déclarées`,async({page})=>{
  const descriptor=await open(page,release),id=descriptor.tool_id,manifest=json('tools/'+id+'/manifest.json'),archive=readFileSync(release.archive);
  expect(sha(archive)).toBe(release.sha256);expect(descriptor.sha256).toBe(release.sha256);
  const results:any[]=[];for(const vector of vectors.filter(v=>v.tool===id)){const output=await action(page,vector.action,vector.input);expect(output).toEqual(vector.output);results.push({action:vector.action,passed:true});}
  expect(results.length).toBeGreaterThan(0);
  // A tool without permissions must never raise a consent prompt, even after running every action.
  if(manifest.permissions.length===0){await expect(page.getByRole('button',{name:'Refuser',exact:true})).toHaveCount(0);}
  matrix.push({tool_id:id,version:descriptor.version,package_sha256:release.sha256,guest_sha256:guestHash(release.archive),permissions:manifest.permissions,actions:results,started:true});
 });
}
