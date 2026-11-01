import type {AiStep} from '../../../web-adapter/src/ai-bridge';
export type Binding={file:number}|{step:number;output:string};
export interface ChatStep {tool_id:string;action_id:string;input:Record<string,unknown>;bind:Record<string,Binding>;requires_consent:true}
export interface Quota {limit:number;used:number;remaining:number;resets_at:number;window_hours:number;service_available:boolean}
export interface ChatAnswer {v:2;reply:string;steps:ChatStep[];quota?:Quota}
export const MAX_FILE=700000;
export type AttachmentKind='image'|'pdf'|'text'|'other';
export function attachmentKind(file:{type:string;name:string}):AttachmentKind{
 if(file.type.startsWith('image/'))return 'image';if(file.type==='application/pdf'||/\.pdf$/i.test(file.name))return 'pdf';
 return file.type.startsWith('text/')||/\.(?:csv|txt|json|md)$/i.test(file.name)?'text':'other';
}
export function checkedAnswer(value:unknown):ChatAnswer{
 const a=value as ChatAnswer;if(!a||a.v!==2||typeof a.reply!=='string'||!a.reply.length||!Array.isArray(a.steps)||a.steps.length>5)throw Error('INVALID_ANSWER');
 for(const s of a.steps)if(!s||typeof s.tool_id!=='string'||!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(s.tool_id)||typeof s.action_id!=='string'||s.requires_consent!==true||!s.input||typeof s.input!=='object'||!s.bind||typeof s.bind!=='object')throw Error('INVALID_ANSWER');
 return a;
}
export function toBase64(bytes:Uint8Array){let text='';for(let i=0;i<bytes.length;i+=32768)text+=String.fromCharCode(...bytes.subarray(i,i+32768));return btoa(text)}
export function fromBase64(value:string){return Uint8Array.from(atob(value),c=>c.charCodeAt(0))}
/** Fills a step from local data only, right before it runs. Nothing here ever reaches the cloud. */
export function resolveStep(step:ChatStep,files:Uint8Array[],outputs:Record<string,unknown>[]):AiStep{
 const input={...step.input};
 for(const [key,binding] of Object.entries(step.bind)){
  if('file' in binding){const file=files[binding.file];if(!file)throw Error('MISSING_FILE');input[key]=key==='data_text'?new TextDecoder().decode(file):toBase64(file)}
  else{const output=outputs[binding.step];if(!output||!(binding.output in output))throw Error('MISSING_OUTPUT');input[key]=output[binding.output]}
 }
 return {tool_id:step.tool_id,action_id:step.action_id,input,requires_consent:true};
}
export const clock=(at:number)=>new Date(at).toLocaleTimeString('fr-FR',{hour:'2-digit',minute:'2-digit'});
const types:Record<string,string>={jpeg:'image/jpeg',png:'image/png',webp:'image/webp'};
/** A binary output is offered as a download: it is the actual result of "reduce this file". */
export function downloads(carry:Record<string,string>,data:Record<string,unknown>,tool:string){
 return Object.entries(carry).filter(([key])=>/_base64$/.test(key)).map(([key,value])=>{const format=typeof data.format==='string'?data.format:key==='pdf_base64'?'pdf':'bin';return {name:tool+'.'+(format==='jpeg'?'jpg':format),type:types[format]??(format==='pdf'?'application/pdf':'application/octet-stream'),value}});
}
