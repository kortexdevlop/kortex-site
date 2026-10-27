import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {createServer} from 'node:net';
import {mkdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'../..');
const probe=createServer();await new Promise(resolveReady=>probe.listen(0,'127.0.0.1',resolveReady));const port=probe.address().port;await new Promise(resolveClosed=>probe.close(resolveClosed));
const child=spawn(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),'dev','--local','--env=','--ip','127.0.0.1','--port',String(port),'--config',resolve(root,'api/wrangler.jsonc'),'--persist-to',resolve(root,'target/api-smoke-state')],{cwd:root,windowsHide:true,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'}});
let output='';child.stdout.on('data',data=>output+=data);child.stderr.on('data',data=>output+=data);
const timer=ms=>new Promise(resolveTimer=>setTimeout(resolveTimer,ms));
try{
 let response;for(let i=0;i<100;i++){
  if(child.exitCode!==null)throw Error('LOCAL_WORKER_EXITED');
  try{response=await fetch(`http://127.0.0.1:${port}/api/v1/health`,{signal:AbortSignal.timeout(1000)});break}catch{await timer(200)}
 }
 assert.ok(response,'LOCAL_WORKER_TIMEOUT');assert.equal(response.status,200);const health=await response.json();assert.deepEqual(health,{status:'ok',api_version:'1'});
 const unknown=await fetch(`http://127.0.0.1:${port}/api/v1/not-found`);assert.equal(unknown.status,404);const error=await unknown.json();assert.equal(error.error.code,'NOT_FOUND');
 mkdirSync(resolve(root,'target'),{recursive:true});const report={local_only:true,remote_writes:0,port,health_status:200,health,unknown_status:404,error_code:error.error.code,headers:{csp:response.headers.get('content-security-policy'),cache:response.headers.get('cache-control')}};
 writeFileSync(resolve(root,'target/api-local-smoke.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report));
}finally{
 if(child.pid&&child.exitCode===null){if(process.platform==='win32')spawnSync('taskkill',['/PID',String(child.pid),'/T','/F'],{windowsHide:true,stdio:'ignore'});else child.kill('SIGTERM')}
 writeFileSync(resolve(root,'target/api-local-smoke.log'),output);
}
