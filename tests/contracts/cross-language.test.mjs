import test from 'node:test';
import Ajv from 'ajv/dist/2020.js';
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { resolve, dirname } from 'node:path';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
const root=fileURLToPath(new URL('../../',import.meta.url));process.chdir(root);
const {validateManifest,validateContract,validateUi,validateWorkflow}=await import('../../shared/typescript/src/index.ts');
const read=p=>JSON.parse(readFileSync(p,'utf8'));
const m=read('shared/fixtures/manifest.valid.json'),c=read('shared/fixtures/contract.valid.json');
const clone=x=>structuredClone(x);
const cases=[];
const add=(name,kind,value,expected)=>cases.push({name,kind,value,expected});
add('manifest canonical','manifest',m,true);add('manifest invalid fixture','manifest',read('shared/fixtures/invalid-manifest.json'),false);
add('contract canonical','contract',c,true);add('contract invalid fixture','contract',read('shared/fixtures/invalid-contract.json'),false);
for(const id of ['ab','UPPER','../escape','a'.repeat(65)])add('slug '+id,'manifest',{...m,id},false);
for(const version of ['01.0.0','1.0','1.0.0-01','1.2.3-alpha.2+build.01'])add('version '+version,'manifest',{...m,version},version==='1.2.3-alpha.2+build.01');
for(const icon_path of ['../x','/x','a/../x','a\\x','https://x','a/./x'])add('path '+icon_path,'manifest',{...m,icon_path},false);
add('unknown property','manifest',{...m,script:'x'},false);
add('duplicate permission','manifest',{...m,permissions:['file.read','file.read']},false);
add('unknown permission','manifest',{...m,permissions:['gpu.direct']},false);
add('Unicode codepoints','manifest',{...m,name:'é'.repeat(100)},true);
add('Unicode over limit','manifest',{...m,name:'😀'.repeat(101)},false);
add('duplicate action','contract',{...c,actions:[c.actions[0],{...c.actions[0],description:'different'}]},false);
for(const [name,schema] of [['remote ref',{type:'object',$ref:'https://example.com'}],['local recursion',{type:'object',$ref:'#'}],['overflow',{type:'number',maximum:1e100}],['inverted',{type:'number',minimum:10,maximum:1}],['missing required',{type:'object',required:['missing']}],['fractional enum',{type:'number',enum:[0.5,1.5]}]]){const value=clone(c);value.actions[0].input_schema=schema;add(name,'contract',value,name==='fractional enum')}
let deep={type:'string'};for(let i=0;i<18;i++)deep={type:'array',items:deep};const nested=clone(c);nested.actions[0].input_schema=deep;add('deep schema','contract',nested,false);
const reorder=x=>Array.isArray(x)?x.map(reorder):x&&typeof x==='object'?Object.fromEntries(Object.entries(x).reverse().map(([k,v])=>[k,reorder(v)])):x;
add('manifest key order','manifest',reorder(m),true);add('contract key order','contract',reorder(c),true);

const protocol = new Ajv({strict:true,allowUnionTypes:true}).compile(read('shared/schemas/protocol.schema.json'));
const header={v:1,request_id:'00000000-0000-4000-8000-000000000001'};
const cap={capability_request_id:'00000000-0000-4000-8000-000000000002',ok:true,data:{}};
for(const [name,value,valid] of [
 ['null data',{...header,ok:true,data:null},false],['null error',{...header,ok:true,error:null},false],
 ['success absent data',{...header,ok:true},true],['success object data',{...header,ok:true,data:{}},true],
 ['missing capability fields',{...header,kind:'capability.result',payload:{}},false],
 ['valid capability',{...header,kind:'capability.result',payload:cap},true],
 ['capability null data',{...header,kind:'capability.result',payload:{...cap,data:null}},false],
 ['capability invalid id',{...header,kind:'capability.result',payload:{...cap,capability_request_id:'bad'}},false],
 ['capability error missing',{...header,kind:'capability.result',payload:{...cap,ok:false}},false],
 ['capability unknown field',{...header,kind:'capability.result',payload:{...cap,unknown:true}},false],
 ['valid capability rejection',{...header,kind:'capability.result',payload:{capability_request_id:cap.capability_request_id,ok:false,error:{code:'DENIED',message:'Denied',retryable:false}}},true],
 ['oversized error code',{...header,ok:false,error:{code:'x'.repeat(101),message:'failed',retryable:false}},false],
 ['invalid capability encode shape',{...header,kind:'capability.result',payload:{...cap,error:null}},false]
])add(name,'protocol',value,valid);

test('TypeScript and Rust agree on the complete shared matrix',()=>{
 const expected=cases.map(x=>x.expected);
 const ts=cases.map(x=>x.kind==='protocol'?protocol(x.value):(x.kind==='manifest'?validateManifest(x.value):validateContract(x.value)).ok);
 assert.deepEqual(ts,expected,'TS expected outcomes');
 const file=resolve('target/contracts/cases.json');mkdirSync(dirname(file),{recursive:true});writeFileSync(file,JSON.stringify(cases));
 const stdout=execFileSync('cargo',['test','-p','kortex-shared','--locked','cross_language_matrix','--','--nocapture'],{cwd:root,env:{...process.env,KORTEX_CONTRACT_CASES:file},encoding:'utf8',maxBuffer:4*1024*1024});
 const match=stdout.match(/KORTEX_MATRIX:(\[[^\n]+\])/);assert.ok(match,'Rust matrix emitted');assert.deepEqual(JSON.parse(match[1]),ts);
 console.log(`${cases.length} common fixture cases accepted/rejected identically`);
});
test('UI and workflow semantic invariants and canonical errors',()=>{
 const ui=read('shared/fixtures/ui.valid.json'),w=read('shared/fixtures/workflow.valid.json');assert.equal(validateUi(ui).ok,true);assert.equal(validateWorkflow(w).ok,true);
 assert.equal(validateUi({...ui,children:[...ui.children,ui.children[0]]}).ok,false);
 assert.equal(validateWorkflow({...w,edges:[...w.edges,{from:'second',to:'first'}]}).ok,false);
 const error=validateManifest({});assert.equal(error.ok,false);assert.ok(error.violations.every(v=>v.code==='SCHEMA'&&typeof v.path==='string'&&v.message==='Schema violation'));
});
test('deterministic regeneration of schema snapshot is byte identical',()=>{
 const names=['manifest','contract','ui','protocol','workflow'];
 const generate=()=>JSON.stringify(Object.fromEntries(names.map(name=>[name,read(`shared/schemas/${name}.schema.json`)])),null,2)+'\n';
 const before=names.map(name=>createHash('sha256').update(readFileSync(`shared/schemas/${name}.schema.json`)).digest('hex'));
 const first=generate(),second=generate();assert.equal(second,first);
 mkdirSync('target/contracts',{recursive:true});writeFileSync('target/contracts/schemas.generated.json',first);writeFileSync('target/contracts/schemas.generated.json',second);
 assert.equal(readFileSync('target/contracts/schemas.generated.json','utf8'),first);
 assert.deepEqual(names.map(name=>createHash('sha256').update(readFileSync(`shared/schemas/${name}.schema.json`)).digest('hex')),before);
});
