import {requireRuntime} from '../../shared/typescript/src/runtime';
import {inspectWasm} from './wasm-check';
import {extractArchive} from './archive';
import {artifactSigningText,type TrustStore} from './trust';
import {matchesSchema} from './browser-contracts';
import {ed25519} from '@noble/curves/ed25519.js';
import {validateManifest,validateContract,type Manifest,type Contract} from '@kortex/contracts';
import {request,reply,capability,error,QUEUE_LIMIT,type Request,type Reply,type CapabilityRequest} from './types';
export interface Descriptor {schema_version:1;tool_id:string;version:string;abi_version:string;sha256:string;signing_key_id:string;signature:string}
const admitted=new WeakSet<object>();
export class VerifiedTool {
 #bytes:Uint8Array;#manifest:Manifest;#contract:Contract;
 private constructor(bytes:Uint8Array,manifest:Manifest,contract:Contract){this.#bytes=bytes.slice();this.#manifest=structuredClone(manifest);this.#contract=structuredClone(contract);admitted.add(this);}
 get bytes(){return this.#bytes.slice()}get manifest(){return structuredClone(this.#manifest)}get contract(){return structuredClone(this.#contract)}
 static async verifyArchive(bytes:Uint8Array,proof:Descriptor,trust:TrustStore):Promise<VerifiedTool>{
  proof=structuredClone(proof);const copy=bytes.slice();await trust.verify(copy,proof);const files=await extractArchive(copy);const decode=(name:string)=>{const b=files.get(name)!;if(b.length>1048576)throw Error('INVALID_METADATA');return JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(b))};
  const m=decode('manifest.json'),c=decode('contract.json');if(!validateManifest(m).ok||!validateContract(c).ok||m.id!==proof.tool_id||m.version!==proof.version||c.tool_id!==m.id)throw Error('INTEGRITY_ERROR');if(m.abi_version!=='1.0')throw Error('UNSUPPORTED_ABI');const allowed=new Set(['manifest.json','contract.json','module.wasm',m.icon_path]);if(!m.icon_path.startsWith('assets/')||!files.get(m.icon_path)?.length||[...files.keys()].some(k=>!allowed.has(k))||c.actions.some((a:any)=>a.permissions.some((p:string)=>!m.permissions.includes(p))))throw Error('INVALID_METADATA');requireRuntime(m);const module=files.get('module.wasm')!;inspectWasm(module);await WebAssembly.compile(module as BufferSource);return new VerifiedTool(module,m,c);
 }
 /// Standalone signed WASM admission for this adapter stage. Metadata must be embedded in signed custom sections.
 /// The .tool ZIP loader is a separate next-stage boundary, never an unsigned development bypass.
 static async verifySignedModule(bytes:Uint8Array,manifest:unknown,contract:unknown,proof:Descriptor,keys:Readonly<Record<string,Uint8Array>>):Promise<VerifiedTool>{
  if(bytes.length>20*1024*1024||!validateManifest(manifest).ok||!validateContract(contract).ok)throw Error('INVALID_METADATA');
  bytes=bytes.slice();manifest=structuredClone(manifest);contract=structuredClone(contract);proof=structuredClone(proof);keys=Object.fromEntries(Object.entries(keys).map(([id,key])=>[id,key.slice()]));
  const m=manifest as Manifest,c=contract as Contract;requireRuntime(m);if(m.abi_version!=='1.0'||proof.abi_version!=='1.0')throw Error('UNSUPPORTED_ABI');
  if(proof.tool_id!==m.id||proof.version!==m.version||c.tool_id!==m.id||!keys[proof.signing_key_id])throw Error('INTEGRITY_ERROR');
  const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes as BufferSource)),b=>b.toString(16).padStart(2,'0')).join('');if(digest!==proof.sha256)throw Error('INTEGRITY_ERROR');

  const text=artifactSigningText(proof);
  let sig:Uint8Array;try{sig=Uint8Array.from(atob(proof.signature),v=>v.charCodeAt(0));}catch{throw Error('INTEGRITY_ERROR')}
  let valid=false;try{const key=await crypto.subtle.importKey('raw',keys[proof.signing_key_id] as BufferSource,{name:'Ed25519'},false,['verify']);valid=await crypto.subtle.verify('Ed25519',key,sig as BufferSource,text);}catch(e){if(!(e instanceof DOMException)||e.name!=='NotSupportedError')throw Error('INTEGRITY_ERROR');valid=ed25519.verify(sig,text,keys[proof.signing_key_id],{zip215:false});}if(!valid)throw Error('INTEGRITY_ERROR');
  const module=await WebAssembly.compile(bytes as BufferSource);
  for(const [name,value] of [['kortex.manifest',m],['kortex.contract',c]] as const){const sections=WebAssembly.Module.customSections(module,name);if(sections.length!==1||JSON.stringify(JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(sections[0])))!==JSON.stringify(value))throw Error('INTEGRITY_ERROR');}
  return new VerifiedTool(bytes.slice(),structuredClone(m),structuredClone(c));
 }
}
export class WebBridge {
 private guestMemory=0;
 /** Passive host diagnostic, not a guest capability or an iframe command. */
 get memoryBytes(){return this.guestMemory}
 private worker:Worker|undefined;private sequence=0;private stopped=false;private pending=new Map<number,{resolve:(v:any)=>void;reject:(e:Error)=>void;timer:ReturnType<typeof setTimeout>}>();private tail:Promise<unknown>=Promise.resolve();private queued=0;
 onCapabilitySettled:()=>Promise<void>=async()=>{};onFailure:(reason:string)=>void=()=>{};
 private capabilities=new Set<Promise<void>>();private capabilityError:Error|undefined;private capabilityVersion=0;private capabilityReply:Reply|undefined;
 capabilityRevision(){return this.capabilityVersion}
 async settleCapabilities(since=this.capabilityVersion):Promise<Reply|undefined>{while(this.capabilities.size)await Promise.allSettled([...this.capabilities]);if(this.stopped)throw Error('STOPPED');if(this.capabilityError)throw this.capabilityError;return this.capabilityVersion>since?this.capabilityReply:undefined}
 onCapability:(call:CapabilityRequest)=>Promise<Record<string,unknown>>=async()=>{throw Error('PERMISSION')};
 private constructor(private tool:VerifiedTool,private timeout:number){
  this.worker=new Worker(new URL('./worker.js',import.meta.url),{type:'module'});
  this.worker.onmessage=({data})=>{if(this.stopped)return;
   if(data?.capability){try{capability(data.capability);if(this.capabilities.size>=QUEUE_LIMIT)throw Error('QUEUE_FULL');const call=data.capability;const job=(async()=>{let outcome:Record<string,unknown>,failure:Error|undefined;try{outcome={ok:true,data:await this.onCapability(call)}}catch(e){failure=e instanceof Error?e:Error('UNAVAILABLE');outcome={ok:false,error:error(failure.message)}}await this.capabilityResult(call.request_id,outcome);if(failure)throw failure})();this.capabilities.add(job);void job.then(()=>this.capabilities.delete(job),e=>{this.capabilityError=e instanceof Error?e:Error('PERMISSION');this.capabilities.delete(job)});}catch{this.stop('INVALID_CAPABILITY')}return;}
   const item=this.pending.get(data?.id);if(!item){this.stop('CORRELATION_ERROR');return;}if(Number.isSafeInteger(data?.memory_bytes)&&data.memory_bytes>=0)this.guestMemory=data.memory_bytes;clearTimeout(item.timer);this.pending.delete(data.id);if(data.error){item.reject(Error(data.error));this.stop(data.error);}else item.resolve(data.result);
  };
  this.worker.onerror=()=>this.stop('GUEST_CRASH');this.worker.onmessageerror=()=>this.stop('INVALID_MESSAGE');
 }
 static async startVerifiedTool(tool:VerifiedTool,timeout=30000):Promise<WebBridge>{
  if(!admitted.has(tool))throw Error('UNVERIFIED_TOOL');if(!Number.isFinite(timeout)||timeout<1||timeout>30000)throw Error('INVALID_TIMEOUT');
  const bridge=new WebBridge(tool,timeout);await bridge.call({op:'start',bytes:tool.bytes.slice()});return bridge;
 }
 private call(message:Record<string,unknown>,duration=this.timeout):Promise<any>{
  if(this.stopped||!this.worker)return Promise.reject(Error('STOPPED'));if(this.pending.size>=QUEUE_LIMIT)return Promise.reject(Error('QUEUE_FULL'));
  const id=++this.sequence;return new Promise((resolve,reject)=>{const timer=setTimeout(()=>this.stop('TIMEOUT'),duration);this.pending.set(id,{resolve,reject,timer});this.worker!.postMessage({id,...message});});
 }
 dispatch(value:Request):Promise<Reply>{
  request(value);const action=value.kind==='action.invoke'?this.tool.contract.actions.find(a=>a.id===value.payload.action_id):undefined;
  if(value.kind==='action.invoke'&&!action)return Promise.reject(Error('UNKNOWN_ACTION'));
  if(action&&!matchesSchema(action.input_schema,value.payload.input))return Promise.reject(Error('INVALID_INPUT'));
  if(this.queued>=QUEUE_LIMIT)return Promise.reject(Error("QUEUE_FULL"));this.queued++;const copy=structuredClone(value);
  const task=this.tail.then(async()=>{const result=await this.call({op:'dispatch',request:copy},action?Math.min(this.timeout,action.timeout_ms):this.timeout);reply(result);if(action&&result.ok&&!matchesSchema(action.output_schema,result.data)){this.stop('INVALID_OUTPUT');throw Error('INVALID_OUTPUT')}if(result.request_id!==copy.request_id){this.stop('CORRELATION_ERROR');throw Error('CORRELATION_ERROR')}return result;});
  this.tail=task.catch(()=>{});return task.finally(()=>{this.queued--;});
 }
 private async capabilityResult(id:string,result:Record<string,unknown>){const reply=await this.dispatch({v:1,request_id:crypto.randomUUID(),kind:'capability.result',payload:{capability_request_id:id,...result}});this.capabilityVersion++;this.capabilityReply=reply;await this.onCapabilitySettled();if(!reply.ok)throw Error(reply.error?.code??'PERMISSION');}
 stop(reason='CANCELLED'){if(this.stopped)return;this.stopped=true;this.worker?.terminate();this.worker=undefined;for(const item of this.pending.values()){clearTimeout(item.timer);item.reject(Error(reason));}this.pending.clear();if(reason!=='CANCELLED')this.onFailure(reason);}
}
export const startVerifiedTool=WebBridge.startVerifiedTool;
