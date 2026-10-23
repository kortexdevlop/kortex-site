import Ajv2020 from 'ajv/dist/2020.js';
import type { ErrorObject, ValidateFunction } from 'ajv';
import manifest from '../../schemas/manifest.schema.json' with { type: 'json' };
import contract from '../../schemas/contract.schema.json' with { type: 'json' };
import ui from '../../schemas/ui.schema.json' with { type: 'json' };
import workflow from '../../schemas/workflow.schema.json' with { type: 'json' };
export interface Violation { code: 'SCHEMA' | 'INVARIANT' | 'LIMIT'; path: string; message: string }
export type Result<T> = { ok: true; value: T } | { ok: false; violations: Violation[] };
const ajv = new Ajv2020({ strict: true, allErrors: true, allowUnionTypes: true });
const validators = { manifest: ajv.compile(manifest), contract: ajv.compile(contract), ui: ajv.compile(ui), workflow: ajv.compile(workflow) };
export type Kind = keyof typeof validators;
function failure(code: Violation['code'], path: string, message: string): Violation { return { code, path, message }; }
function bounded(node: any, depth = 0): boolean {
  if (depth > 16) return false;
  if (node.minimum !== undefined && node.maximum !== undefined && node.minimum > node.maximum) return false;
  for (const [min,max] of [['minLength','maxLength'],['minItems','maxItems']]) if(node[min] !== undefined && node[max] !== undefined && node[min] > node[max]) return false;
  if (node.required?.some((key: string) => !Object.hasOwn(node.properties || {}, key))) return false;
  if (node.enum?.some((x: unknown) => typeof x === 'number' && (!Number.isFinite(x) || Math.abs(x) > Number.MAX_SAFE_INTEGER))) return false;
  return [...Object.values(node.properties || {}), ...(node.items ? [node.items] : [])].every(child => bounded(child, depth + 1));
}
function invariant(kind: Kind, value: any): boolean {
  if (kind === 'contract') return new Set(value.actions.map((a:any)=>a.id)).size === value.actions.length && value.actions.every((a:any)=>bounded(a.input_schema)&&bounded(a.output_schema));
  if (kind === 'ui') {
    const keys=new Set();let count=0;
    function visit(n:any,depth=0):boolean { if(depth>32||++count>2000||keys.has(n.key))return false;keys.add(n.key);if(n.type==='svg-chart'&&n.props.series.some((s:any)=>s.values.length!==n.props.labels.length))return false;return n.children.every((c:any)=>visit(c,depth+1)); }
    return visit(value);
  }
  if (kind === 'workflow') {
    const ids=new Set<string>(value.nodes.map((n:any)=>n.id));if(ids.size!==value.nodes.length)return false;
    const graph=new Map<string,string[]>([...ids].map(id=>[id,[]]));const edges=new Set<string>();
    for(const e of value.edges){const k=JSON.stringify([e.from,e.to]);if(!ids.has(e.from)||!ids.has(e.to)||edges.has(k))return false;edges.add(k);graph.get(e.from)!.push(e.to);}
    for(const n of value.nodes)for(const b of Object.values(n.input_bindings) as any[])if(b.from_node&&!edges.has(JSON.stringify([b.from_node,n.id])))return false;
    const active=new Set<string>(),done=new Set<string>();
    function visit(id:string):boolean { if(active.has(id))return false;if(done.has(id))return true;active.add(id);if(!graph.get(id)!.every(visit))return false;active.delete(id);done.add(id);return true; }
    return [...ids].every(visit);
  }
  return true;
}
export function validate<T>(kind: Kind, value: unknown): Result<T> {
  try {
    const bytes=JSON.stringify(value);
    if(bytes===undefined || new TextEncoder().encode(bytes).length>1048576) return {ok:false,violations:[failure('LIMIT','','JSON document exceeds 1 MiB')]};
    const validator: ValidateFunction=validators[kind];
    if(!validator(value))return {ok:false,violations:(validator.errors || []).map((e:ErrorObject)=>failure('SCHEMA',e.instancePath,'Schema violation')).sort((a,b)=>a.path.localeCompare(b.path))};
    if(!invariant(kind,value))return {ok:false,violations:[failure('INVARIANT','','Semantic invariant violated')]};
    return {ok:true,value:value as T};
  } catch { return {ok:false,violations:[failure('LIMIT','','Invalid JSON document')]}; }
}
