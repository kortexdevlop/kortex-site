import {PermissionBroker} from './permissions';
import {IndexedToolStorage,storageKey} from './storage';
import {capability,bounded,type CapabilityRequest} from './types';
const FILE_LIMIT=10*1024*1024,CHUNK_LIMIT=256*1024;
function base64(bytes:Uint8Array){let value='';for(let i=0;i<bytes.length;i+=8192)value+=String.fromCharCode(...bytes.subarray(i,i+8192));return btoa(value);}
function decode(value:unknown){if(typeof value!=='string'||value.length>1400000)throw Error('MESSAGE_LIMIT');let raw:string;try{raw=atob(value)}catch{throw Error('INVALID_INPUT')};return Uint8Array.from(raw,v=>v.charCodeAt(0));}
function networkOrigin(input:unknown):URL {
 if(typeof input!=='string')throw Error('NETWORK_DENIED');let u:URL;try{u=new URL(input)}catch{throw Error('NETWORK_DENIED')};const host=u.hostname.toLowerCase();
 if(u.protocol!=='https:'||u.username||u.password||u.hash||host==='localhost'||host.endsWith('.localhost')||host.endsWith('.local')||host.includes(':'))throw Error('NETWORK_DENIED');
 if(/^\d+(?:\.\d+){3}$/.test(host)){const [a,b,c,d]=host.split('.').map(Number);if([a,b,c,d].some(n=>n>255)||a===0||a===10||a===127||a>=224||a===169&&b===254||a===172&&b>=16&&b<=31||a===192&&b===168||a===100&&b>=64&&b<=127||a===192&&b===0&&[0,2].includes(c)||a===198&&[18,19].includes(b)||a===198&&b===51&&c===100||a===203&&b===0&&c===113)throw Error('NETWORK_DENIED')}
 return u;
}
export class BrowserCapabilities {
 readonly permissions:PermissionBroker;private handles=new Map<string,{file:File;expires:number}>();private active=new Set<AbortController>();private closed=false;private epoch=0;private files=new Set<()=>void>();
 readonly storage:IndexedToolStorage;
 constructor(tool_id:string,version:string,declared:readonly string[],options:{profile:string;storageVersion:number;dedicatedOrigin:string;parentOrigin:string;networkOrigins:readonly string[]},changed:()=>void=()=>{}){
  this.permissions=new PermissionBroker(tool_id,version,declared,changed);this.storage=new IndexedToolStorage(options.profile,tool_id,options.storageVersion,options.dedicatedOrigin,options.parentOrigin);
  this.origins=new Set(options.networkOrigins.map(v=>{const u=networkOrigin(v);if(u.origin!==v)throw Error('NETWORK_DENIED');return v}));
 }
 private origins:Set<string>;
 private scope(call:CapabilityRequest){capability(call);const p=call.payload;
  if(call.capability.startsWith('storage.')){storageKey(String(p.key??''));if(p.secret===true)throw Error('PERMISSION');return String(p.key);}
  if(call.capability==='network.fetch'){const u=networkOrigin(p.url);if(!this.origins.has(u.origin))throw Error('NETWORK_DENIED');if(!/^\d+(?:\.\d+){3}$/.test(u.hostname))throw Error('UNSUPPORTED');return u.origin;}
  if(call.capability==='file.read'){if(typeof p.handle!=='string'||!/^[a-f0-9]{64}$/.test(p.handle))throw Error('INVALID_HANDLE');return p.handle;}
  return call.capability;
 }
 async request(call:CapabilityRequest){const scope=this.scope(call);return this.permissions.ask(call.capability,scope,gesture=>this.execute(call,gesture));}
 async execute(call:CapabilityRequest,gesture?:object):Promise<Record<string,unknown>>{
  if(this.closed)throw Error('CANCELLED');const scope=this.scope(call);const expires=this.permissions.authorize(call.capability,scope),epoch=this.epoch;
  const p=call.payload;
  if(call.capability==='file.pick'){this.permissions.consumeGesture(gesture,call.capability);return this.pick();}
  if(call.capability==='file.save'){
   this.permissions.consumeGesture(gesture,call.capability);const bytes=decode(p.data_base64);if(bytes.length>FILE_LIMIT)throw Error('SIZE_LIMIT');const name=typeof p.name==='string'?p.name:'download.bin';if(name.length>256||/[\\/\x00-\x1f]/.test(name))throw Error('INVALID_INPUT');
   const url=URL.createObjectURL(new Blob([bytes]));const a=document.createElement('a');a.download=name;a.href=url;a.hidden=true;document.body.append(a);a.click();a.remove();setTimeout(()=>URL.revokeObjectURL(url),1000);return {saved:true};
  }
  if(call.capability==='clipboard.write'){
   this.permissions.consumeGesture(gesture,call.capability);if(typeof p.text!=='string'||new TextEncoder().encode(p.text).length>1048000)throw Error('MESSAGE_LIMIT');if(!navigator.clipboard?.writeText)throw Error('UNSUPPORTED');
   try{await navigator.clipboard.writeText(p.text)}catch{throw Error('PERMISSION')}if(this.closed||epoch!==this.epoch||Date.now()>=expires)throw Error('CANCELLED');return {written:true};
  }
  const controller=new AbortController();this.active.add(controller);const timeout=setTimeout(()=>controller.abort(),30000);
  try{
   if(call.capability.startsWith('storage.')&&gesture){this.permissions.consumeGesture(gesture,call.capability);await this.storage.requestPersistence();}
   let result:Record<string,unknown>;
   switch(call.capability){
    case 'storage.read':result={value:await this.storage.get(scope,controller.signal)};break;
    case 'storage.write':result=p.operation==='delete'?await this.storage.delete(scope,controller.signal):await this.storage.set(scope,p.value,controller.signal);break;
    case 'file.read':{
     const handle=this.handles.get(scope);if(!handle||handle.expires<=Date.now())throw Error('INVALID_HANDLE');
     const offset=p.offset??0,length=p.length??CHUNK_LIMIT;if(!Number.isSafeInteger(offset)||!Number.isSafeInteger(length)||(offset as number)<0||(length as number)<1||(length as number)>CHUNK_LIMIT)throw Error('INVALID_INPUT');
     const bytes=new Uint8Array(await handle.file.slice(offset as number,(offset as number)+(length as number)).arrayBuffer());result={data_base64:base64(bytes),offset,eof:(offset as number)+bytes.length>=handle.file.size};break;
    }
    case 'network.fetch':{
     if(!navigator.onLine)throw Error('OFFLINE');let response:Response;
     try{response=await fetch(String(p.url),{method:'GET',redirect:'error',credentials:'omit',referrerPolicy:'no-referrer',cache:'no-store',mode:'cors',signal:controller.signal})}catch{throw Error(controller.signal.aborted?'CANCELLED':'UNAVAILABLE')}
     if(!response.ok)throw Error('UNAVAILABLE');if(Number(response.headers.get('content-length'))>CHUNK_LIMIT){await response.body?.cancel();throw Error('SIZE_LIMIT')}
     const reader=response.body?.getReader();const parts:Uint8Array[]=[];let size=0;if(reader){try{for(;;){const {done,value}=await reader.read();if(done)break;size+=value.length;if(size>CHUNK_LIMIT){await reader.cancel();throw Error('SIZE_LIMIT')}parts.push(value)}}finally{reader.releaseLock()}}
     const bytes=new Uint8Array(size);let at=0;for(const part of parts){bytes.set(part,at);at+=part.length}result={status:response.status,data_base64:base64(bytes)};break;
    }
    default:throw Error('UNSUPPORTED');
   }
   if(this.closed||epoch!==this.epoch||Date.now()>=expires||controller.signal.aborted)throw Error('CANCELLED');bounded(result);return result;
  }finally{clearTimeout(timeout);this.active.delete(controller)}
 }
 private pick():Promise<Record<string,unknown>>{
  return new Promise((resolve,reject)=>{const input=document.createElement('input');input.type='file';input.hidden=true;let finished=false;
   const cancel=()=>finish(Error('CANCELLED'));const finish=(error?:Error,result?:Record<string,unknown>)=>{if(finished)return;finished=true;clearTimeout(timer);this.files.delete(cancel);input.remove();if(error)reject(error);else resolve(result!)};
   const timer=setTimeout(cancel,60000);this.files.add(cancel);input.oncancel=cancel;
   input.onchange=()=>{const file=input.files?.[0];if(!file){cancel();return}if(file.size>FILE_LIMIT){finish(Error('SIZE_LIMIT'));return}if(this.handles.size>=128){finish(Error('QUOTA'));return}const token=Array.from(crypto.getRandomValues(new Uint8Array(32)),b=>b.toString(16).padStart(2,'0')).join('');this.handles.set(token,{file,expires:Date.now()+300000});finish(undefined,{handle:token,name:file.name,size:file.size})};
   document.body.append(input);try{input.click()}catch{finish(Error('PERMISSION'))}
  });
 }
 revoke(){this.epoch++;this.permissions.revoke();this.handles.clear();for(const controller of this.active)controller.abort();for(const cancel of this.files)cancel();}
 close(){this.closed=true;this.revoke();this.permissions.close();this.storage.close();}
}
