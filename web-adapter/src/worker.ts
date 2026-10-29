import {bounded,request,reply,capability,LIMIT,QUEUE_LIMIT,type Request,type Reply,type CapabilityRequest} from './types';
import {inspectWasm} from './wasm-check';
import {range,unpack,inspectMemory} from './memory';
type Guest={memory:WebAssembly.Memory;kortex_alloc:(n:number)=>number;kortex_dealloc:(p:number,n:number)=>void;kortex_dispatch:(p:number,n:number)=>bigint};
const port=globalThis as unknown as {postMessage:(v:unknown)=>void;onmessage:(event:MessageEvent)=>void};
let guest:Guest|undefined;let busy=false;let queued=0;const pending=new Set<string>();
function dispatch(r:Request):Reply {
 if(!guest)throw Error('UNAVAILABLE');request(r);
 const bytes=bounded(r),input=guest.kortex_alloc(bytes.length)>>>0;
 let output=0,length=0,validOutput=false;
 try {
  range(guest.memory,input,bytes.length);new Uint8Array(guest.memory.buffer,input,bytes.length).set(bytes);
  [output,length]=unpack(guest.kortex_dispatch(input,bytes.length));range(guest.memory,output,length);
  if(output<input+bytes.length&&input<output+length)throw Error('OVERLAPPING_BUFFER');validOutput=true;
  const data=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array(guest.memory.buffer,output,length).slice()));reply(data);if(data.request_id!==r.request_id)throw Error('CORRELATION_ERROR');
  if(r.kind==='capability.result'){pending.delete(String(r.payload.capability_request_id));queued=Math.max(0,queued-1);}
  return data;
 }finally{if(validOutput)guest.kortex_dealloc(output,length);guest.kortex_dealloc(input,bytes.length);}
}
port.onmessage=async({data})=>{
 if(busy){port.postMessage({id:data?.id,error:'QUEUE_BUSY'});return;}busy=true;
 try{
  if(!data||!Number.isSafeInteger(data.id)||!['start','dispatch'].includes(data.op))throw Error('INVALID_MESSAGE');
  let result:unknown;
  if(data.op==='start'){
   if(guest||!(data.bytes instanceof Uint8Array)||data.bytes.length>20*1024*1024)throw Error('INVALID_MODULE');
   inspectWasm(data.bytes);inspectMemory(data.bytes);const module=await WebAssembly.compile(data.bytes);
   if(WebAssembly.Module.imports(module).some(i=>i.module!=='kortex_v1'||i.name!=='host_enqueue'||i.kind!=='function'))throw Error('FORBIDDEN_IMPORT');
   const exports=WebAssembly.Module.exports(module);for(const [name,kind] of [['memory','memory'],['kortex_alloc','function'],['kortex_dealloc','function'],['kortex_dispatch','function']])if(!exports.some(e=>e.name===name&&e.kind===kind))throw Error('INVALID_EXPORTS');
   const instance=await WebAssembly.instantiate(module,{kortex_v1:{host_enqueue:(p:number,n:number)=>{
    if(!guest)return -1;
    try{range(guest.memory,p>>>0,n>>>0);const call:unknown=JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array(guest.memory.buffer,p>>>0,n>>>0)));capability(call);
     if(queued>=QUEUE_LIMIT||pending.has(call.request_id))return -2;queued++;pending.add(call.request_id);port.postMessage({capability:call});return 0;
    }catch{return -1;}
   }}});guest=instance.exports as Guest;result={ready:true};
  }else{request(data.request);if(data.request.kind==='capability.result'&&!pending.has(String(data.request.payload.capability_request_id)))throw Error('UNKNOWN_CAPABILITY_RESULT');result=dispatch(data.request);}
  port.postMessage({id:data.id,result,memory_bytes:guest?.memory.buffer.byteLength??0});
 }catch(e){port.postMessage({id:data?.id,error:e instanceof Error?e.message:'ENGINE_ERROR'});}finally{busy=false;}
};
