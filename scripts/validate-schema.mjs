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
