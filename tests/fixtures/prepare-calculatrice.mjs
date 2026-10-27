import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'../..');
const env={...process.env,RUSTFLAGS:'-C link-arg=--max-memory=67108864'};
for(const [exe,args] of [['cargo',['build','-p','tool-calculatrice','--target','wasm32-unknown-unknown','--release','--locked']],['node',['tests/fixtures/calculatrice.cjs']]]){
 const result=spawnSync(exe,args,{cwd:root,env,stdio:'inherit'});if(result.status!==0)process.exit(result.status??1);
}
