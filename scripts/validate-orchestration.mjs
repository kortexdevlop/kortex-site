import assert from 'node:assert/strict';import {spawn} from 'node:child_process';import {writeFileSync,readdirSync} from 'node:fs';
const root=process.cwd();
async function check(name,bin,args,cwd=root){console.log('Checking '+name);const result=await new Promise(resolve=>{const child=spawn(bin,args,{cwd,windowsHide:true,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'}});let output='';child.stdout.on('data',v=>output+=v);child.stderr.on('data',v=>output+=v);child.on('error',e=>resolve({code:-1,output:String(e)}));child.on('close',code=>resolve({code,output}))});writeFileSync('target/orchestration-'+name+'.log',result.output);assert.equal(result.code,0,name+' failed; see its local log');console.log('PASS '+name)}
await check('api-types',process.execPath,['node_modules/typescript/bin/tsc','--noEmit','-p','api/tsconfig.json']);
await check('adapter-types',process.execPath,['node_modules/typescript/bin/tsc','--noEmit','-p','web-adapter/tsconfig.json']);
await check('web-tests',process.execPath,['--experimental-strip-types','--test',...readdirSync('website/tests').filter(f=>f.endsWith('.test.ts')).map(f=>'website/tests/'+f)]);
await check('rust-runner','cargo',['test','-p','kortex-workflows','--locked','--offline']);
await check('native-store','cargo',['test','-p','kortex-desktop','--lib','workflow','--locked','--offline']);
await check('native-file','cargo',['test','-p','kortex-desktop','--test','workflow_files','--locked','--offline']);
await check('api-tests',process.execPath,['../node_modules/vitest/vitest.mjs','run'],root+'/api');
await check('web-check',process.execPath,['website/node_modules/astro/bin/astro.mjs','check','--root','website']);
await check('web-build',process.execPath,['website/node_modules/astro/bin/astro.mjs','build','--root','website']);
console.log('Local orchestration checks passed');
