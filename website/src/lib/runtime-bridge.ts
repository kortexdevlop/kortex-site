/** Deployment configuration only: never derive the runtime origin from a query or message. */
export function runtimeOrigin(value:string|undefined,parent:string):string|null {
  if(!value)return null;
  try{const url=new URL(value),site=new URL(parent);if(url.origin!==value||url.origin===site.origin||url.username||url.password)return null;
    if(url.protocol==='https:'&&['run.kortex.site','run.staging.kortex.site','kortex-run-staging.userpgl.workers.dev'].includes(url.hostname))return url.origin;
    if(['127.0.0.1','localhost'].includes(site.hostname)&&['127.0.0.1','localhost'].includes(url.hostname)&&url.protocol==='http:')return url.origin;
  }catch{}return null;
}
export function connectRuntime(frame:HTMLIFrameElement,origin:string,id:string,version:string,changed:(state:string)=>void):()=>void {
  if(runtimeOrigin(origin,location.origin)!==origin)throw Error('Runtime origin rejected');
  const nonce=Array.from(crypto.getRandomValues(new Uint8Array(32)),b=>b.toString(16).padStart(2,'0')).join('');
  let session:string|undefined,closed=false,finished=false;
  const hello=()=>{if(!closed&&!session)frame.contentWindow?.postMessage({kind:'HELLO',nonce,client_version:'1.0'},origin);};
  const receive=(event:MessageEvent)=>{
    if(closed||finished||event.origin!==origin||event.source!==frame.contentWindow)return;
    const data=event.data;if(!data||typeof data!=='object'||Array.isArray(data)||data.nonce!==nonce)return;
    if(data.kind==='READY'&&!session&&typeof data.session_id==='string'&&/^[a-f0-9-]{36}$/.test(data.session_id)){
      session=data.session_id;clearInterval(retry);changed('loading');frame.contentWindow!.postMessage({kind:'OPEN',nonce,session_id:session,tool_id:id,version},origin);
    }else if(session&&data.session_id===session){
      if(data.kind==='RESULT'&&data.result?.tool_id===id&&data.result?.version===version&&data.result?.status==='ready'){finished=true;clearTimeout(deadline);changed('ready');}
      else if(data.kind==='ERROR'){finished=true;clearTimeout(deadline);changed('error');}
    }
  };
  const retry=setInterval(hello,500),deadline=setTimeout(()=>{finished=true;clearInterval(retry);changed('error');},20000);
  window.addEventListener('message',receive);frame.addEventListener('load',hello);changed('connecting');hello();
  return ()=>{closed=true;clearInterval(retry);clearTimeout(deadline);window.removeEventListener('message',receive);frame.removeEventListener('load',hello);};
}
