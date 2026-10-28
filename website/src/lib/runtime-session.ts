import {runtimeOrigin} from './runtime-bridge.ts';
export type RuntimeStatus='loading'|'ready'|'busy'|'error'|'closed';
/** One fixed tool/version, source window and nonce. This channel never accepts commands from guests. */
export function runtimeSession(frame:HTMLIFrameElement,origin:string,tool:string,version:string,changed:(state:RuntimeStatus)=>void=()=>{}){
 if(runtimeOrigin(origin,location.origin)!==origin)throw Error('ORIGIN');
 const nonce=Array.from(crypto.getRandomValues(new Uint8Array(32)),b=>b.toString(16).padStart(2,'0')).join('');let session='',opened=false,closed=false,count=0;
 let resolveReady:()=>void=()=>{},rejectReady:(e:Error)=>void=()=>{};const ready=new Promise<void>((a,b)=>{resolveReady=a;rejectReady=b});void ready.catch(()=>{});
 let pending:{id:string;resolve:(v:unknown)=>void;reject:(e:Error)=>void;timer:ReturnType<typeof setTimeout>;cleanup:()=>void}|null=null;
 const send=(data:Record<string,unknown>)=>frame.contentWindow?.postMessage({...data,nonce,session_id:session},origin);
 function stop(reason='CANCELLED',state:RuntimeStatus='closed'){if(closed)return;closed=true;clearInterval(retry);clearTimeout(deadline);window.removeEventListener('message',receive);rejectReady(Error(reason));if(pending){clearTimeout(pending.timer);pending.cleanup();pending.reject(Error(reason));pending=null}frame.src='about:blank';changed(state)}
 function receive(e:MessageEvent){if(closed||e.source!==frame.contentWindow||e.origin!==origin||!e.data||e.data.nonce!==nonce)return;const d=e.data;
  if(d.kind==='READY'&&!session&&typeof d.session_id==='string'&&/^[a-f0-9-]{36}$/.test(d.session_id)){session=d.session_id;send({kind:'OPEN',tool_id:tool,version})}
  else if(session&&d.session_id===session){if(d.kind==='RESULT'&&!opened&&d.result?.tool_id===tool&&d.result.version===version&&d.result.status==='ready'){opened=true;clearInterval(retry);clearTimeout(deadline);resolveReady();changed('ready')}
   else if(d.kind==='AI_RESULT'&&pending&&d.request_id===pending.id&&d.result?.executed_locally===true&&d.result.shareable===false){const p=pending;pending=null;clearTimeout(p.timer);p.cleanup();changed('ready');p.resolve(d.result.data)}
   else if(d.kind==='ERROR')stop('UNAVAILABLE','error');else if(d.kind==='AI_ERROR'&&pending&&d.request_id===pending.id)stop('PERMISSION','error');}
 }
 const hello=()=>{if(!session)send({kind:'HELLO',client_version:'1.0'})},retry=setInterval(hello,300),deadline=setTimeout(()=>stop('UNAVAILABLE','error'),30000);
 window.addEventListener('message',receive);frame.src=origin+'/frame.html?tool='+encodeURIComponent(tool)+'&version='+encodeURIComponent(version);changed('loading');hello();
 return{ready,close:()=>stop(),async invoke(action:string,input:unknown,signal:AbortSignal){await ready;if(closed)throw Error('CANCELLED');signal.throwIfAborted();if(pending)throw Error('BUSY');if(count>=50)throw Error('LIMIT');count++;return new Promise<unknown>((resolve,reject)=>{const id=crypto.randomUUID(),abort=()=>stop('CANCELLED','error');pending={id,resolve,reject,timer:setTimeout(()=>stop('TIMEOUT','error'),60000),cleanup:()=>signal.removeEventListener('abort',abort)};signal.addEventListener('abort',abort,{once:true});changed('busy');if(signal.aborted)abort();else send({kind:'WORKFLOW_INVOKE',request_id:id,action_id:action,input})})}};
}
