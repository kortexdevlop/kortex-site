import fs from 'node:fs';
import assert from 'node:assert/strict';
import Ajv from 'ajv/dist/2020.js';
import addFormats from 'ajv-formats';
const read=p=>JSON.parse(fs.readFileSync(p,'utf8'));
const ajv=new Ajv({strict:true,allErrors:true,allowUnionTypes:true});addFormats(ajv);
export const validateManifest=ajv.compile(read('shared/schemas/manifest.schema.json'));
const shape=ajv.compile(read('shared/schemas/contract.schema.json'));
function boundedSchema(node, depth=0, seen=new Set()) {
  if(depth>16 || seen.has(node)) return false;
  seen.add(node);
  if (node.minimum!==undefined && node.maximum!==undefined && node.minimum>node.maximum) return false;
  for(const [min,max] of [['minLength','maxLength'],['minItems','maxItems']]) if(node[min]!==undefined && node[max]!==undefined && node[min]>node[max]) return false;
  if(node.required?.some(key=>!Object.hasOwn(node.properties||{},key)))return false;
  if(node.enum?.some(x=>typeof x==='number' && (!Number.isFinite(x) || Math.abs(x)>Number.MAX_SAFE_INTEGER))) return false;
  const children=[...Object.values(node.properties||{}),...(node.items?[node.items]:[])];
  const ok=children.every(child=>boundedSchema(child,depth+1,seen));seen.delete(node);return ok;
}
export function validateContract(value) {
  try {
    if(!shape(value))return false;
    const ids=value.actions.map(a=>a.id);
    return new Set(ids).size===ids.length && value.actions.every(a=>boundedSchema(a.input_schema)&&boundedSchema(a.output_schema));
  } catch { return false; }
}
const m=read('shared/fixtures/manifest.valid.json'),c=read('shared/fixtures/contract.valid.json');
const checks=[];const check=(label,fn)=>{assert.ok(fn(),label);checks.push(label)};
check('valid manifest',()=>validateManifest(m));check('valid contract',()=>validateContract(c));
check('invalid manifest',()=>!validateManifest(read('shared/fixtures/invalid-manifest.json')));
check('invalid contract',()=>!validateContract(read('shared/fixtures/invalid-contract.json')));
for(const id of ['ab','../abc','UPPER','a'.repeat(65)])check('invalid slug '+id,()=>!validateManifest({...m,id}));
for(const version of ['01.0.0','1.0','1.0.0-01'])check('invalid semver '+version,()=>!validateManifest({...m,version}));
for(const icon_path of ['../x','/x','a/../x','a\\x','https://x','a/./x'])check('invalid path '+icon_path,()=>!validateManifest({...m,icon_path}));
check('duplicate permissions',()=>!validateManifest({...m,permissions:['file.read','file.read']}));
check('unknown permission',()=>!validateManifest({...m,permissions:['gpu.direct']}));
check('duplicate actions',()=>!validateContract({...c,actions:[c.actions[0],{...c.actions[0],description:'different'}]}));
const actionSchema=s=>({...c,actions:[{...c.actions[0],input_schema:s}]});
check('remote ref',()=>!validateContract(actionSchema({type:'object',$ref:'https://example.com/schema'})));
check('local guest recursion',()=>!validateContract(actionSchema({type:'object',$ref:'#'})));
check('overflow bound',()=>!validateContract(actionSchema({type:'number',maximum:1e100})));
check('inverted bounds',()=>!validateContract(actionSchema({type:'string',minLength:20,maxLength:1})));
let deep={type:'string'};for(let i=0;i<18;i++)deep={type:'array',items:deep};
check('excessive depth',()=>!validateContract(actionSchema(deep)));
const reorder=x=>Array.isArray(x)?x.map(reorder):x&&typeof x==='object'?Object.fromEntries(Object.entries(x).reverse().map(([k,v])=>[k,reorder(v)])):x;
check('key order independent',()=>validateManifest(reorder(m))&&validateContract(reorder(c)));
console.log(`${checks.length} schema checks passed; AJV validators compiled from JSON Schema 2020-12`);

// Structural schemas plus graph/tree invariants, shared-package extraction in 2.4.
const uiShape=ajv.compile(read('shared/schemas/ui.schema.json'));
const workflowShape=ajv.compile(read('shared/schemas/workflow.schema.json'));
const protocolShape=ajv.compile(read('shared/schemas/protocol.schema.json'));
export function validateUI(root) {
  if(!uiShape(root))return false;
  const keys=new Set();let count=0;
  function visit(node,depth=0){
    if(depth>32 || ++count>2000 || keys.has(node.key))return false;keys.add(node.key);
    if(node.type==='svg-chart' && node.props.series.some(s=>s.values.length!==node.props.labels.length))return false;
    return node.children.every(child=>visit(child,depth+1));
  }
  return visit(root);
}
export function validateWorkflow(value) {
  if(!workflowShape(value))return false;
  const ids=new Set(value.nodes.map(n=>n.id));if(ids.size!==value.nodes.length)return false;
  const edges=new Set();const graph=new Map([...ids].map(id=>[id,[]]));
  for(const e of value.edges){const key=JSON.stringify([e.from,e.to]);if(!ids.has(e.from)||!ids.has(e.to)||edges.has(key))return false;edges.add(key);graph.get(e.from).push(e.to)}
  for(const node of value.nodes)for(const binding of Object.values(node.input_bindings))if(binding.from_node && !edges.has(JSON.stringify([binding.from_node,node.id])))return false;
  const active=new Set(),done=new Set();
  function visit(id){if(active.has(id))return false;if(done.has(id))return true;active.add(id);if(!graph.get(id).every(visit))return false;active.delete(id);done.add(id);return true}
  return [...ids].every(visit);
}
const ui=read('shared/fixtures/ui.valid.json'),workflow=read('shared/fixtures/workflow.valid.json');
check('rich UI valid',()=>validateUI(ui));check('workflow valid',()=>validateWorkflow(workflow));
check('duplicate UI key',()=>!validateUI({...ui,children:[...ui.children,ui.children[0]]}));
check('unknown node',()=>!validateUI({...ui,type:'script'}));
check('CSS URL rejected',()=>!validateUI({...ui,props:{style:'background:url(https://example.com)'}}));
check('raw HTML rejected',()=>!validateUI({...ui,props:{html:'<script />'}}));
check('workflow cycle',()=>!validateWorkflow({...workflow,edges:[...workflow.edges,{from:'second',to:'first'}]}));
check('duplicate workflow node',()=>!validateWorkflow({...workflow,nodes:[...workflow.nodes,workflow.nodes[0]]}));
check('missing source node',()=>!validateWorkflow({...workflow,edges:[{from:'missing',to:'second'}]}));
const actionOutput=ajv.compile(c.actions[0].output_schema);
check('output outside action schema',()=>!actionOutput({result:'wrong'}));
check('reply protocol',()=>protocolShape({v:1,request_id:'00000000-0000-4000-8000-000000000001',ok:false,error:{code:'FAILED',message:'Failed',retryable:false}}));
console.log(`${checks.length} total schema checks passed (UI, workflow, protocol included)`);
check('chart lengths',()=>!validateUI({type:'svg-chart',key:'badchart',props:{kind:'bar',labels:['A'],series:[{label:'x',values:[]}]},children:[]}));
check('tree count',()=>!validateUI({...ui,children:Array.from({length:2000},(_,i)=>({type:'text',key:'n'+i,props:{text:'x'},children:[]}))}));
const cap={v:1,request_id:'00000000-0000-4000-8000-000000000001',kind:'capability.result',payload:{capability_request_id:'00000000-0000-4000-8000-000000000002',ok:true,data:{}}};
check('capability result',()=>protocolShape(cap));
check('inconsistent capability result',()=>!protocolShape({...cap,payload:{...cap.payload,ok:false}}));
console.log(`${checks.length} final schema checks passed`);
