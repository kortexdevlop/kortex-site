import {Revocations} from './revocation';
import type {RootKey} from './trust';
import {ToolLoader,type ArtifactDescriptor} from './loader';
import {TrustStore} from './trust';
import {BrowserCapabilities} from './capabilities';
import {WebBridge,VerifiedTool} from './bridge';
import type {Manifest} from '@kortex/contracts';
import type {Request} from './types';
import {AiReview} from './ai-bridge';
import {matchesSchema} from './browser-contracts';
export interface RuntimeWebConfig {api_origin:string;allowed_parent_origin:string;tool_id:string;tool_version:string;network_origins?:readonly string[];revocation_root?:RootKey}
export class BrowserHost {
 private aiReview:AiReview|undefined;private aiTool:VerifiedTool|undefined;
 private workflowPermissions:Set<string>|undefined;
 reviewAi(value:unknown){if(!this.bridge||!this.aiTool)throw Error('UNAVAILABLE');this.aiReview=new AiReview(value,this.aiTool,this.bridge)}
 async decideAi(index:number,approved:boolean){if(!this.aiReview)throw Error('NO_REVIEW');return this.aiReview.decide(index,approved)}
 async invokeWorkflow(actionId:unknown,input:unknown){
  const tool=this.aiTool,bridge=this.bridge,generation=this.generation;
  if(!tool||!bridge||typeof actionId!=='string')throw Error('UNAVAILABLE');
  const action=tool.contract.actions.find(a=>a.id===actionId);
  if(!action||new TextEncoder().encode(JSON.stringify(input)).length>1048576||!matchesSchema(action.input_schema,input))throw Error('INVALID_INPUT');
  await bridge.settleCapabilities();const checkpoint=bridge.capabilityRevision();this.workflowPermissions=new Set(action.permissions);
  let result;try{const initial=await bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind:'action.invoke',payload:{action_id:actionId,input:structuredClone(input)}});result=await bridge.settleCapabilities(checkpoint)??initial;}finally{this.workflowPermissions=undefined}
  if(generation!==this.generation||!result.ok)throw Error('ACTION_FAILED');
  if(new TextEncoder().encode(JSON.stringify(result.data)).length>1048576||!matchesSchema(action.output_schema,result.data))throw Error('INVALID_OUTPUT');
  return{executed_locally:true,data:result.data,shareable:false};
 }
 protected bridge:WebBridge|undefined;private generation=0;private loading:AbortController|undefined;protected capabilities:BrowserCapabilities|undefined;
 constructor(){document.addEventListener('click',event=>{const button=event.target instanceof Element?event.target.closest('button[data-host-approval]'):null;if(!button||!this.capabilities)return;try{this.capabilities.permissions.respond(button.getAttribute('data-host-approval')!,button.getAttribute('data-host-decision')!,event)}catch{/* synthetic/untrusted approval refused */}},true);}
protected current:Record<string,unknown>={status:navigator.onLine?'unavailable':'offline'};
 protected emit(){window.dispatchEvent(new Event('kortex:host'))}
 snapshot(){const current={...structuredClone(this.current),pending:this.capabilities?.permissions.prompts()??[]};return navigator.onLine?current:{...current,status:'offline'}}
 async loadTool(descriptor:ArtifactDescriptor,trust:TrustStore,config:RuntimeWebConfig){this.stop();const generation=this.generation;const controller=new AbortController();this.loading=controller;const loader=new ToolLoader(trust,config.api_origin,status=>{if(generation===this.generation){this.current={status,config};this.emit()}},config.revocation_root?new Revocations(config.revocation_root):undefined);try{const tool=await loader.loadTool(descriptor,controller.signal);if(generation!==this.generation)throw Error('CANCELLED');await this.startVerifiedTool(tool,config)}catch(e){if(generation===this.generation){this.current={status:'error'};this.emit()}throw e}}
 async uninstall(descriptor:ArtifactDescriptor,trust:TrustStore,config:RuntimeWebConfig){this.stop();await new ToolLoader(trust,config.api_origin).uninstall(descriptor)}
 async startVerifiedTool(tool:VerifiedTool,config:RuntimeWebConfig){
  for(const origin of [config.api_origin,config.allowed_parent_origin]){const u=new URL(origin);if(u.origin!==origin||!(u.protocol==='https:'||u.protocol==='http:'&&['localhost','127.0.0.1'].includes(u.hostname)))throw Error('INVALID_CONFIG');}
  if(config.tool_id!==tool.manifest.id||config.tool_version!==tool.manifest.version)throw Error('INVALID_CONFIG');
  this.stop();this.aiTool=tool;const generation=this.generation;this.current={status:'loading',config};this.emit();
  try{const bridge=await WebBridge.startVerifiedTool(tool);if(generation!==this.generation){bridge.stop();throw Error('CANCELLED')}this.bridge=bridge;this.capabilities=new BrowserCapabilities(tool.manifest.id,tool.manifest.version,tool.manifest.permissions,{profile:'local',storageVersion:1,dedicatedOrigin:location.origin,parentOrigin:config.allowed_parent_origin,networkOrigins:config.network_origins??[]},()=>this.emit());const services=this.capabilities;bridge.onCapability=call=>{if(this.workflowPermissions&&!this.workflowPermissions.has(call.capability))return Promise.reject(Error('PERMISSION'));return services.request(call)};bridge.onFailure=()=>{if(generation===this.generation){services.close();this.current={status:'error'};this.emit();}};bridge.onCapabilitySettled=async()=>{if(generation!==this.generation)return;const rendered=await this.dispatch('render',{});if(rendered.ok){this.current={...this.current,ui:rendered.data};this.emit();}};const init=await this.dispatch('init',{});if(!init.ok)throw Error(init.error?.code||'INIT_FAILED');const result=await this.dispatch('render',{});if(!result.ok)throw Error(result.error?.code||'GUEST_ERROR');this.current={status:'ready',config,ui:result.data};this.emit();}catch(e){if(generation===this.generation){this.stop();this.current={status:'error'};this.emit();}throw e;}
 }
 protected async dispatch(kind:Request['kind'],payload:Record<string,unknown>){if(!this.bridge)throw Error('UNAVAILABLE');return this.bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind,payload});}
 async command(message:Record<string,unknown>){
  try{if(message.kind==='revoke'){this.capabilities?.revoke();this.emit();return;}if(message.kind==='stop'){this.stop();return;}if(message.kind==='ui.event'){await this.dispatch('ui.event',message.payload as Record<string,unknown>);const result=await this.dispatch('render',{});this.current={...this.current,ui:result.data};}
  else if(message.kind==='action.invoke'){const result=await this.dispatch('action.invoke',message.payload as Record<string,unknown>);this.current={...this.current,result:result.ok?result.data:result.error};}
  this.emit();}catch{this.current={status:'error'};this.emit();}
 }
 stop(){this.aiReview=undefined;this.aiTool=undefined;this.generation++;this.loading?.abort();this.loading=undefined;this.capabilities?.close();this.capabilities=undefined;this.bridge?.stop();this.bridge=undefined;this.current={status:'unavailable'};this.emit();}
}
declare global {interface Window {kortexHost:BrowserHost}}
if(!window.kortexHost){window.kortexHost=new BrowserHost();window.dispatchEvent(new Event('kortex:host'));}
export {VerifiedTool,WebBridge,startVerifiedTool} from './bridge';

export {ToolLoader,TrustStore};
