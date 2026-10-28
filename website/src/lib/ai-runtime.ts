import {runtimeOrigin} from './runtime-bridge';
import type {AiStep} from '../../../web-adapter/src/ai-bridge';
export function executeLocal(frame:HTMLIFrameElement,origin:string,step:AiStep,version:string,signal:AbortSignal):Promise<unknown>{
 if(runtimeOrigin(origin,location.origin)!==origin)throw Error('INVALID_ORIGIN');
 const nonce=Array.from(crypto.getRandomValues(new Uint8Array(32)),b=>b.toString(16).padStart(2,'0')).join('');let session='',review='',execution='',opened=false;
 return new Promise((resolve,reject)=>{
  const send=(data:Record<string,unknown>)=>frame.contentWindow?.postMessage({...data,nonce,session_id:session},origin);
  const cleanup=()=>{window.removeEventListener('message',receive);clearInterval(retry);clearTimeout(timer);signal.removeEventListener('abort',abort)};
  const fail=()=>{cleanup();reject(Error('L’action a été refusée ou interrompue.'))},abort=()=>fail();
  const receive=(event:MessageEvent)=>{if(event.origin!==origin||event.source!==frame.contentWindow)return;const d=event.data;if(!d||d.nonce!==nonce)return;if(d.kind==='READY'&&!session&&typeof d.session_id==='string'){session=d.session_id;send({kind:'OPEN',tool_id:step.tool_id,version})}else if(d.session_id===session&&session){if(d.kind==='RESULT'&&!opened&&d.result?.tool_id===step.tool_id&&d.result?.version===version){opened=true;review=crypto.randomUUID();send({kind:'AI_REVIEW',request_id:review,plan:{v:1,summary:'Action approuvée localement',steps:[step]}})}else if(d.kind==='AI_RESULT'&&d.request_id===review&&d.result?.reviewed){review='';execution=crypto.randomUUID();send({kind:'AI_DECIDE',request_id:execution,index:0,approved:true})}else if(d.kind==='AI_RESULT'&&d.request_id===execution&&d.result?.executed_locally===true&&d.result.shareable===false){cleanup();resolve(d.result.data)}else if(['ERROR','AI_ERROR'].includes(d.kind))fail()}};
  const hello=()=>{if(!session)send({kind:'HELLO',client_version:'1.0'})},retry=setInterval(hello,300),timer=setTimeout(fail,60000);window.addEventListener('message',receive);signal.addEventListener('abort',abort,{once:true});if(signal.aborted)abort();else hello();
 });
}
