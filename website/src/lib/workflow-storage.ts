import {decode,type Workflow} from './workflows.ts';
const privateKey=/^(?:data_base64|content_base64|data_text|bytes|file_path|path|handle|file_handle|content)$/i;
export function persistentWorkflow(value:unknown):Workflow{
 const w=decode(value);const safe=(v:unknown,depth=0):boolean=>{if(depth>32)return false;if(Array.isArray(v))return v.every(x=>safe(x,depth+1));if(v&&typeof v==='object')return Object.entries(v).every(([k,x])=>(!privateKey.test(k)||k==='data_base64'&&x==='')&&safe(x,depth+1));return typeof v!=='string'||!/(?:^[A-Z]:\\|^\/home\/|^\/Users\/)/i.test(v)};
 for(const n of w.nodes)for(const[p,b]of Object.entries(n.input_bindings))if('value'in b&&(p.split('/').some(k=>privateKey.test(k)&&!(k==='data_base64'&&b.value===''))||!safe(b.value)))throw Error('Retirez les contenus de fichiers et les chemins avant de sauvegarder.');return w;
}
export const workflowStorageKey='kortex.workflows.v1';
export function savedWorkflows(storage:Pick<Storage,'getItem'>):Workflow[]{try{const raw=storage.getItem(workflowStorageKey);if(!raw||raw.length>4194304)return[];const values=JSON.parse(raw);if(!Array.isArray(values)||values.length>20)return[];return values.map(persistentWorkflow)}catch{return[]}}
export function saveWorkflow(storage:Pick<Storage,'getItem'|'setItem'>,value:unknown){const w=persistentWorkflow(value),items=savedWorkflows(storage).filter(x=>x.id!==w.id);items.push(w);if(items.length>20||new TextEncoder().encode(JSON.stringify(items)).length>4194304)throw Error('Limite locale : 20 parcours et 4 Mo.');storage.setItem(workflowStorageKey,JSON.stringify(items));return items}
