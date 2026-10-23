import fs from 'node:fs';
import assert from 'node:assert/strict';
import { fileURLToPath, pathToFileURL } from 'node:url';
import Ajv from 'ajv/dist/2020.js';
import { validateManifest, validateContract } from '../../shared/typescript/src/index.ts';
const root=new URL('../../',import.meta.url);
const read=path=>JSON.parse(fs.readFileSync(new URL(path,root),'utf8'));
const ajv=new Ajv({strict:true,allErrors:true,allowUnionTypes:true});
const protocol=ajv.compile(read('shared/schemas/protocol.schema.json'));
const validVersion=ajv.compile(read('shared/schemas/manifest.schema.json').properties.version);
function semver(text){
 if(!validVersion(text))throw Error('Invalid SemVer');
 const match=/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/.exec(text);
 if(!match)throw Error('Invalid SemVer');return {numbers:match.slice(1,4).map(BigInt),pre:match[4]?.split('.')};
}
function compare(a,b){
 a=semver(a);b=semver(b);for(let i=0;i<3;i++)if(a.numbers[i]!==b.numbers[i])return a.numbers[i]>b.numbers[i]?1:-1;
 if(!a.pre||!b.pre)return a.pre?-1:b.pre?1:0;
 for(let i=0;i<Math.max(a.pre.length,b.pre.length);i++){
  const x=a.pre[i],y=b.pre[i];if(x===y)continue;if(x===undefined)return -1;if(y===undefined)return 1;
  const nx=/^\d+$/.test(x),ny=/^\d+$/.test(y);if(nx&&ny)return BigInt(x)>BigInt(y)?1:-1;if(nx!==ny)return nx?-1:1;return x>y?1:-1;
 }return 0;
}
export function negotiate(tool,runtime){
 for(const [field,list,code] of [['abi_version','supported_abi','UNSUPPORTED_ABI'],['format_version','supported_format','UNSUPPORTED_FORMAT'],['contract_version','supported_contract','UNSUPPORTED_CONTRACT'],['ui_schema_version','supported_ui','UNSUPPORTED_UI']])if(!runtime[list]?.includes(tool[field]))return code;
 try {if(compare(runtime.version,tool.min_runtime)<0)return 'RUNTIME_TOO_OLD';}catch{return 'INVALID_VERSION';}return 'ACCEPT';
}
// Specification oracle over captured messages. It does not instantiate or mock a runtime.
export function evaluate(testCase,contract){
 const admission=negotiate(testCase.tool,testCase.runtime);if(admission!=='ACCEPT')return admission;
 let initialized=false,closed=false;const ids=new Set();
 for(const {request,response} of testCase.messages){
  if(!protocol(request)||!protocol(response))return 'INVALID_PROTOCOL';
  if(ids.has(request.request_id))return 'DUPLICATE_REQUEST';ids.add(request.request_id);
  if(response.request_id!==request.request_id)return 'CORRELATION_ERROR';
  if(closed||(!initialized&&request.kind!=='init')||(initialized&&request.kind==='init'))return 'INVALID_SEQUENCE';
  if(request.kind==='init'){if(!response.ok)return 'INIT_FAILED';initialized=true;}
  if(request.kind==='shutdown')closed=true;
  if(request.kind==='action.invoke'){
   const action=contract.actions.find(a=>a.id===request.payload.action_id);if(!action)return 'UNKNOWN_ACTION';
   if(!ajv.compile(action.input_schema)(request.payload.input))return 'INVALID_INPUT';
   if(response.ok&&!ajv.compile(action.output_schema)(response.data))return 'INVALID_OUTPUT';
  }
 }
 if(testCase.capability){const c=testCase.capability;if(!['storage.read','storage.write','file.pick','file.read','file.save','network.fetch','clipboard.write'].includes(c.name)||!c.declared.includes(c.name)||!c.granted.includes(c.name))return 'PERMISSION_DENIED';}
 return 'ACCEPT';
}
export function checkCorpus(corpus){
 assert.equal(corpus.corpus_version,'1.0');assert.ok(validateManifest(corpus.manifest).ok);assert.ok(validateContract(corpus.contract).ok);
 const names=new Set();for(const testCase of corpus.cases){assert.ok(!names.has(testCase.name),'duplicate case name');names.add(testCase.name);assert.equal(evaluate(testCase,corpus.contract),testCase.expected,testCase.name);}
 return corpus.cases.length;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){const corpus=read('tests/conformance/v1.cases.json');console.log(`${checkCorpus(corpus)} conformance cases passed (specification corpus; cross-host pending)`);}
