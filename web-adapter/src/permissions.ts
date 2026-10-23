import type {CapabilityRequest} from './types';
import {CAPABILITIES,QUEUE_LIMIT} from './types';
export interface PermissionPrompt {id:string;tool_id:string;version:string;instance:string;capability:string;scope:string;expires:number}
type Grant={expires:number;once:boolean};
export class PermissionBroker {
 readonly instance=crypto.randomUUID();private grants=new Map<string,Grant>();private denied=new Set<string>();private pending=new Map<string,{prompt:PermissionPrompt;perform:(gesture?:object)=>Promise<Record<string,unknown>>;resolve:(v:Record<string,unknown>)=>void;reject:(e:Error)=>void;timer:ReturnType<typeof setTimeout>}>();private gestures=new WeakMap<object,{cap:string;expires:number}>();private closed=false;
 constructor(readonly tool_id:string,readonly version:string,private declared:readonly string[],private changed:()=>void=()=>{}){this.declared=[...declared];}
 prompts(){return Array.from(this.pending.values(),v=>({...v.prompt}));}
 private key(cap:string,scope:string){if(this.closed||!CAPABILITIES.includes(cap as any)||!this.declared.includes(cap))throw Error('PERMISSION');return `${cap}\u0000${scope}`;}
 authorize(cap:string,scope:string){const key=this.key(cap,scope),grant=this.grants.get(key);if(this.denied.has(key)||!grant||grant.expires<=Date.now())throw Error('PERMISSION');if(grant.once)this.grants.delete(key);return grant.expires;}
 ask(cap:string,scope:string,perform:(gesture?:object)=>Promise<Record<string,unknown>>):Promise<Record<string,unknown>>{
  const key=this.key(cap,scope);if(this.denied.has(key))return Promise.reject(Error('PERMISSION'));
  const grant=this.grants.get(key);const needsGesture=['file.pick','file.save','clipboard.write'].includes(cap);
  if(grant&&grant.expires>Date.now()&&!needsGesture)return perform();
  if(this.pending.size>=QUEUE_LIMIT)return Promise.reject(Error('QUEUE_FULL'));
  return new Promise((resolve,reject)=>{const id=crypto.randomUUID();const prompt={id,tool_id:this.tool_id,version:this.version,instance:this.instance,capability:cap,scope,expires:Date.now()+60000};const timer=setTimeout(()=>{this.pending.delete(id);reject(Error('CANCELLED'));this.changed()},60000);this.pending.set(id,{prompt,perform,resolve,reject,timer});this.changed();});
 }
 respond(id:string,decision:string,event:Event){
  const entry=this.pending.get(id);if(!entry)return;
  if(!(event instanceof Event)||!event.isTrusted||!['click','keydown'].includes(event.type))throw Error('PERMISSION');
  this.pending.delete(id);clearTimeout(entry.timer);const {capability:cap,scope}=entry.prompt,key=this.key(cap,scope);
  if(decision==='deny'||entry.prompt.expires<=Date.now()){this.denied.add(key);entry.reject(Error('PERMISSION'));this.changed();return;}
  if(!['once','session'].includes(decision)){entry.reject(Error('PERMISSION'));this.changed();return;}
  this.grants.set(key,{expires:Date.now()+300000,once:decision==='once'});
  let gesture:object|undefined;if(['file.pick','file.save','clipboard.write','storage.read','storage.write'].includes(cap)){gesture={};this.gestures.set(gesture,{cap,expires:Date.now()+1000});}
  // Start gesture-sensitive APIs synchronously within the trusted host button event.
  try{entry.perform(gesture).then(entry.resolve,entry.reject);}catch(e){entry.reject(e instanceof Error?e:Error('UNAVAILABLE'));}this.changed();
 }
 consumeGesture(token:object|undefined,cap:string){const value=token&&this.gestures.get(token);if(!value||value.cap!==cap||value.expires<Date.now()||!navigator.userActivation?.isActive)throw Error('PERMISSION');this.gestures.delete(token!);}
 revoke(){this.grants.clear();for(const entry of this.pending.values()){clearTimeout(entry.timer);entry.reject(Error('CANCELLED'));}this.pending.clear();this.gestures=new WeakMap();this.changed();}
 close(){this.closed=true;this.revoke();}
}
