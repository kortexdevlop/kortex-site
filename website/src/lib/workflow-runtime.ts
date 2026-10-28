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
/** Each invocation gets a dedicated, signed runtime; removing its frame terminates its worker. */
export function browserExecutor(container:HTMLElement,origin:string):LocalExecutor{
 if(runtimeOrigin(origin,location.origin)!==origin)throw Error('INVALID_ORIGIN');let active:HTMLIFrameElement|null=null,stop:(()=>void)|null=null,closed=false;
 return{close(){closed=true;stop?.();active?.remove();active=null},async invoke(entry,input,signal){if(closed||signal.aborted)throw Error('CANCELLED');if(active)throw Error('BUSY');
  const frame=document.createElement('iframe');active=frame;frame.title='Outil local : '+entry.tool_name;frame.src=origin+'/frame.html?tool='+encodeURIComponent(entry.tool_id)+'&version='+encodeURIComponent(entry.version);frame.setAttribute('sandbox','allow-scripts allow-same-origin');container.append(frame);
  const nonce=Array.from(crypto.getRandomValues(new Uint8Array(32)),b=>b.toString(16).padStart(2,'0')).join('');let session='',request='',opened=false;
  return new Promise((resolve,reject)=>{const send=(d:Record<string,unknown>)=>frame.contentWindow?.postMessage({...d,nonce,session_id:session},origin);
   const cleanup=()=>{clearInterval(retry);clearTimeout(timer);window.removeEventListener('message',receive);signal.removeEventListener('abort',abort);frame.remove();active=null;stop=null};
   const abort=()=>{cleanup();reject(Error('CANCELLED'))};stop=abort;
   const fail=()=>{cleanup();reject(Error('LOCAL_ACTION_REFUSED'))};
   const receive=(e:MessageEvent)=>{if(e.source!==frame.contentWindow||e.origin!==origin||!e.data||e.data.nonce!==nonce)return;const d=e.data;if(d.kind==='READY'&&!session&&typeof d.session_id==='string'&&/^[a-f0-9-]{36}$/.test(d.session_id)){session=d.session_id;send({kind:'OPEN',tool_id:entry.tool_id,version:entry.version})}else if(session&&d.session_id===session){if(d.kind==='RESULT'&&!opened&&d.result?.status==='ready'&&d.result.tool_id===entry.tool_id&&d.result.version===entry.version){opened=true;request=crypto.randomUUID();send({kind:'WORKFLOW_INVOKE',request_id:request,action_id:entry.action.id,input})}else if(d.kind==='AI_RESULT'&&request&&d.request_id===request&&d.result?.executed_locally===true&&d.result.shareable===false){cleanup();resolve(d.result.data)}else if(['ERROR','AI_ERROR'].includes(d.kind))fail()}};
   const hello=()=>{if(!session)send({kind:'HELLO',client_version:'1.0'})},retry=setInterval(hello,300),timer=setTimeout(fail,60000);window.addEventListener('message',receive);signal.addEventListener('abort',abort,{once:true});if(signal.aborted)abort();else hello();
  });
 }};
}
