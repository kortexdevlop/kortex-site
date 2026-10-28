import {runtimeOrigin} from './runtime-bridge.ts';
import {assembleInput,validate,type Workflow,type Registry,type Entry} from './workflows.ts';
import {matchesValueSchema} from '../../../shared/typescript/src/value-schema.ts';
export interface RunEvent{node_id:string;status:'running'|'completed'|'failed';at_ms:number}
export interface LocalExecutor{invoke(entry:Entry,input:unknown,signal:AbortSignal):Promise<unknown>;close():void}
export class LocalRunError extends Error{outputs:Map<string,unknown>;constructor(message:string,outputs:Map<string,unknown>){super(message);this.outputs=outputs}}
export async function runWorkflow(value:Workflow,registry:Registry,executor:LocalExecutor,signal:AbortSignal,consent:(node:Workflow['nodes'][number],entry:Entry)=>Promise<boolean>,progress:(event:RunEvent)=>void=()=>{}){
 const checked=validate(value,registry);if(!checked.ok||!value.nodes.length)throw Error(checked.ok?'EMPTY':checked.message);
 const frozen=structuredClone(checked.workflow),specs=structuredClone(registry),outputs=new Map<string,unknown>();let total=0;
 const deadline=AbortSignal.any([signal,AbortSignal.timeout(300000)]),abort=()=>executor.close();deadline.addEventListener('abort',abort,{once:true});
 try{for(const id of checked.order){deadline.throwIfAborted();const node=frozen.nodes.find(n=>n.id===id)!,entry=specs.get(node.action_ref)!;
  if(node.requires_confirmation||entry.action.permissions.length||entry.action.side_effect!=='none'){const approved=await new Promise<boolean>((resolve,reject)=>{const abort=()=>reject(Error('CANCELLED'));deadline.addEventListener('abort',abort,{once:true});Promise.resolve(consent(node,entry)).then(resolve,reject).finally(()=>deadline.removeEventListener('abort',abort));if(deadline.aborted)abort()});if(!approved)throw Error('CONSENT_DENIED');deadline.throwIfAborted()}
  const input=assembleInput(node,entry,outputs);progress({node_id:id,status:'running',at_ms:Date.now()});
  try{const data=await executor.invoke(entry,input,deadline);deadline.throwIfAborted();const size=new TextEncoder().encode(JSON.stringify(data)).length;total+=size;if(size>1048576||total>4194304||!matchesValueSchema(entry.action.output_schema as unknown as Record<string,unknown>,data))throw Error('INVALID_OUTPUT');outputs.set(id,structuredClone(data));progress({node_id:id,status:'completed',at_ms:Date.now()})}catch(e){progress({node_id:id,status:'failed',at_ms:Date.now()});throw e}
 }return outputs;}catch(e){throw new LocalRunError(e instanceof Error?e.message:'LOCAL_ACTION_REFUSED',structuredClone(outputs))}finally{deadline.removeEventListener('abort',abort);executor.close()}
}
/** A run retains at most eight isolated signed tool instances, including their local handles. */
export function browserExecutor(container:HTMLElement,origin:string):LocalExecutor{
 if(runtimeOrigin(origin,location.origin)!==origin)throw Error('INVALID_ORIGIN');let closed=false;
 type Channel={frame:HTMLIFrameElement;invoke(entry:Entry,input:unknown,signal:AbortSignal):Promise<unknown>;close():void};const channels=new Map<string,Channel>();
 function channel(entry:Entry,signal:AbortSignal):Channel{
  const frame=document.createElement('iframe');frame.title='Outil local : '+entry.tool_name;frame.src=origin+'/frame.html?tool='+encodeURIComponent(entry.tool_id)+'&version='+encodeURIComponent(entry.version);frame.setAttribute('sandbox','allow-scripts allow-same-origin');frame.style.width='100%';frame.style.height='420px';container.append(frame);
  const nonce=Array.from(crypto.getRandomValues(new Uint8Array(32)),b=>b.toString(16).padStart(2,'0')).join('');let session='',opened=false,stopped=false;
  let readyResolve:()=>void=()=>{},readyReject:(e:Error)=>void=()=>{};const ready=new Promise<void>((resolve,reject)=>{readyResolve=resolve;readyReject=reject});void ready.catch(()=>{});
  let pending:{id:string;resolve:(v:unknown)=>void;reject:(e:Error)=>void;timer:ReturnType<typeof setTimeout>}|null=null;
  const send=(data:Record<string,unknown>)=>frame.contentWindow?.postMessage({...data,nonce,session_id:session},origin);
  const fail=(reason='LOCAL_ACTION_REFUSED')=>{if(stopped)return;stopped=true;clearInterval(retry);clearTimeout(openTimer);window.removeEventListener('message',receive);signal.removeEventListener('abort',abort);readyReject(Error(reason));if(pending){clearTimeout(pending.timer);pending.reject(Error(reason));pending=null}frame.remove()};const abort=()=>fail('CANCELLED');
  const receive=(e:MessageEvent)=>{if(stopped||e.source!==frame.contentWindow||e.origin!==origin||!e.data||e.data.nonce!==nonce)return;const d=e.data;if(d.kind==='READY'&&!session&&typeof d.session_id==='string'&&/^[a-f0-9-]{36}$/.test(d.session_id)){session=d.session_id;send({kind:'OPEN',tool_id:entry.tool_id,version:entry.version})}else if(session&&d.session_id===session){if(d.kind==='RESULT'&&!opened&&d.result?.status==='ready'&&d.result.tool_id===entry.tool_id&&d.result.version===entry.version){opened=true;clearInterval(retry);clearTimeout(openTimer);readyResolve()}else if(d.kind==='AI_RESULT'&&pending&&d.request_id===pending.id&&d.result?.executed_locally===true&&d.result.shareable===false){clearTimeout(pending.timer);const resolve=pending.resolve;pending=null;resolve(d.result.data)}else if(d.kind==='ERROR'||d.kind==='AI_ERROR'&&pending&&d.request_id===pending.id)fail()}};
  const hello=()=>{if(!session)send({kind:'HELLO',client_version:'1.0'})},retry=setInterval(hello,300),openTimer=setTimeout(()=>fail(),60000);window.addEventListener('message',receive);signal.addEventListener('abort',abort,{once:true});if(signal.aborted)abort();else hello();
  return{frame,close:()=>fail('CANCELLED'),async invoke(spec,input,signal){await ready;signal.throwIfAborted();if(stopped||pending)throw Error('LOCAL_ACTION_REFUSED');return new Promise((resolve,reject)=>{const id=crypto.randomUUID();pending={id,resolve,reject,timer:setTimeout(()=>fail(),60000)};send({kind:'WORKFLOW_INVOKE',request_id:id,action_id:spec.action.id,input})})}};
 }
 return{close(){closed=true;for(const c of channels.values())c.close();channels.clear()},async invoke(entry,input,signal){if(closed||signal.aborted)throw Error('CANCELLED');const key=entry.tool_id+'@'+entry.version;let current=channels.get(key);if(!current){if(channels.size>=8)throw Error('Limite : huit outils distincts par parcours.');current=channel(entry,signal);channels.set(key,current)}for(const c of channels.values())c.frame.hidden=c!==current;return current.invoke(entry,input,signal)}};
}
