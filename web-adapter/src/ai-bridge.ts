import {matchesSchema} from './browser-contracts';
import type {VerifiedTool,WebBridge} from './bridge';
export interface AiStep {tool_id:string;action_id:string;input:Record<string,unknown>;requires_consent:true}
export interface AiPlan {v:1;steps:AiStep[];summary:string}
export function checkedPlan(value:unknown):AiPlan{
 const p=value as AiPlan;if(!p||typeof p!=='object'||Object.keys(p).sort().join(',')!=='steps,summary,v'||p.v!==1||typeof p.summary!=='string'||!p.summary.length||p.summary.length>500||!Array.isArray(p.steps)||p.steps.length>10||new TextEncoder().encode(JSON.stringify(p)).length>32768)throw Error('INVALID_PLAN');
 for(const s of p.steps)if(!s||typeof s!=='object'||Object.keys(s).sort().join(',')!=='action_id,input,requires_consent,tool_id'||typeof s.tool_id!=='string'||!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(s.tool_id)||s.tool_id.length>64||typeof s.action_id!=='string'||!s.action_id.length||s.action_id.length>64||s.requires_consent!==true||!s.input||typeof s.input!=='object'||Array.isArray(s.input))throw Error('INVALID_PLAN');
 return structuredClone(p);
}
export function localResult(value:unknown):unknown{
 if(new TextEncoder().encode(JSON.stringify(value)).length>32768)throw Error('INVALID_OUTPUT');
 const scrub=(v:unknown,d=0):unknown=>{if(d>16)return null;if(Array.isArray(v))return v.map(x=>scrub(x,d+1));if(v&&typeof v==='object')return Object.fromEntries(Object.entries(v).map(([k,x])=>[k,/^(?:handle|file_handle|path|file_path|data_base64|content_base64|bytes)$/.test(k)?'[masqué]':scrub(x,d+1)]));if(typeof v==='string'&&/(?:[A-Z]:\\|^\/home\/|^\/Users\/)/i.test(v))return '[chemin masqué]';return v};return scrub(value);
}
export class AiReview {
 #plan:AiPlan;#used=new Set<number>();#expires:number;
 constructor(value:unknown,private tool:VerifiedTool,private bridge:WebBridge,private now=()=>Date.now()){
  this.#plan=checkedPlan(value);this.#expires=now()+300000;
  for(const s of this.#plan.steps){const a=tool.contract.actions.find(a=>a.id===s.action_id);if(s.tool_id!==tool.manifest.id||!a||!matchesSchema(a.input_schema,s.input))throw Error('UNKNOWN_ACTION_OR_INPUT')}
 }
 async decide(index:number,approved:boolean){if(this.now()>=this.#expires)throw Error('PLAN_EXPIRED');const s=this.#plan.steps[index];if(!s||this.#used.has(index))throw Error('ALREADY_DECIDED');this.#used.add(index);if(!approved)throw Error('DENIED');const a=this.tool.contract.actions.find(a=>a.id===s.action_id);if(!a||!matchesSchema(a.input_schema,s.input))throw Error('INVALID_INPUT');
  const result=await this.bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind:'action.invoke',payload:{action_id:s.action_id,input:structuredClone(s.input)}});if(!result.ok)throw Error(result.error?.code??'ACTION_FAILED');return {executed_locally:true as const,data:localResult(result.data),shareable:false as const};
 }
}
