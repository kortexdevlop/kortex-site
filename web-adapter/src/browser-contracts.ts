import {manifest,contract} from './generated-validators';
import type {Manifest,Contract} from '@kortex/contracts';
export type {Manifest,Contract};
const wrap=<T>(validator:(v:unknown)=>boolean,value:unknown)=>{try{return new TextEncoder().encode(JSON.stringify(value)).length<=1048576&&validator(value)?{ok:true as const,value:value as T}:{ok:false as const}}catch{return{ok:false as const}}};
export const validateManifest=(value:unknown)=>wrap<Manifest>(manifest,value);
export function validateContract(value:unknown){const result=wrap<Contract>(contract,value);if(!result.ok)return result;const ids=new Set<string>();for(const action of result.value.actions){if(ids.has(action.id)||!bounded(action.input_schema,0)||!bounded(action.output_schema,0))return {ok:false as const};ids.add(action.id)}return result;}
function bounded(s:any,depth:number):boolean{return depth<=16&&!(s.minimum!==undefined&&s.maximum!==undefined&&s.minimum>s.maximum)&&!(s.minLength!==undefined&&s.maxLength!==undefined&&s.minLength>s.maxLength)&&!(s.minItems!==undefined&&s.maxItems!==undefined&&s.minItems>s.maxItems)&&!(s.required??[]).some((k:string)=>!Object.hasOwn(s.properties??{},k))&&!(s.enum??[]).some((v:unknown)=>typeof v==='number'&&(!Number.isFinite(v)||Math.abs(v)>Number.MAX_SAFE_INTEGER))&&Object.values(s.properties??{}).every(v=>bounded(v,depth+1))&&(!s.items||bounded(s.items,depth+1));}
export function matchesSchema(s:any,v:any):boolean{
 const type=v===null?'null':Array.isArray(v)?'array':typeof v;if(s.type==='integer'?!Number.isInteger(v):type!==s.type)return false;
 if(s.enum&&!s.enum.some((x:any)=>Object.is(x,v)))return false;
 if(type==='number'&&(!Number.isFinite(v)||(s.minimum!==undefined&&v<s.minimum)||(s.maximum!==undefined&&v>s.maximum)))return false;
 if(type==='string'){const n=Array.from(v).length;if(s.minLength!==undefined&&n<s.minLength||s.maxLength!==undefined&&n>s.maxLength)return false;}
 if(type==='array'){if(s.minItems!==undefined&&v.length<s.minItems||s.maxItems!==undefined&&v.length>s.maxItems)return false;if(s.items&&!v.every((x:any)=>matchesSchema(s.items,x)))return false;}
 if(type==='object'&&v!==null){if((s.required??[]).some((k:string)=>!Object.hasOwn(v,k)))return false;for(const k of Object.keys(v)){if(Object.hasOwn(s.properties??{},k)){if(!matchesSchema(s.properties[k],v[k]))return false;}else if(s.additionalProperties===false)return false;}}
 return true;
}
