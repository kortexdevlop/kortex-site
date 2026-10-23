import {WebBridge,VerifiedTool} from './bridge';
import type {Manifest} from '@kortex/contracts';
import type {Request} from './types';
export interface RuntimeWebConfig {api_origin:string;allowed_parent_origin:string;tool_id:string;tool_version:string}
export class BrowserHost {
 protected bridge:WebBridge|undefined;private generation=0;protected current:Record<string,unknown>={status:navigator.onLine?'unavailable':'offline'};
 protected emit(){window.dispatchEvent(new Event('kortex:host'))}
 snapshot(){return navigator.onLine?structuredClone(this.current):{...structuredClone(this.current),status:'offline'}}
 async startVerifiedTool(tool:VerifiedTool,config:RuntimeWebConfig){
  for(const origin of [config.api_origin,config.allowed_parent_origin]){const u=new URL(origin);if(u.origin!==origin||!(u.protocol==='https:'||u.protocol==='http:'&&['localhost','127.0.0.1'].includes(u.hostname)))throw Error('INVALID_CONFIG');}
  if(config.tool_id!==tool.manifest.id||config.tool_version!==tool.manifest.version)throw Error('INVALID_CONFIG');
  this.stop();const generation=this.generation;this.current={status:'loading',config};this.emit();
  try{const bridge=await WebBridge.startVerifiedTool(tool);if(generation!==this.generation){bridge.stop();throw Error('CANCELLED')}this.bridge=bridge;const init=await this.dispatch('init',{});if(!init.ok)throw Error(init.error?.code||'INIT_FAILED');const result=await this.dispatch('render',{});if(!result.ok)throw Error(result.error?.code||'GUEST_ERROR');this.current={status:'ready',config,ui:result.data};this.emit();}catch(e){if(generation===this.generation){this.stop();this.current={status:'error'};this.emit();}throw e;}
 }
 protected async dispatch(kind:Request['kind'],payload:Record<string,unknown>){if(!this.bridge)throw Error('UNAVAILABLE');return this.bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind,payload});}
 async command(message:Record<string,unknown>){
  try{if(message.kind==='stop'){this.stop();return;}if(message.kind==='ui.event'){await this.dispatch('ui.event',message.payload as Record<string,unknown>);const result=await this.dispatch('render',{});this.current={...this.current,ui:result.data};}
  else if(message.kind==='action.invoke'){const result=await this.dispatch('action.invoke',message.payload as Record<string,unknown>);this.current={...this.current,result:result.ok?result.data:result.error};}
  this.emit();}catch{this.current={status:'error'};this.emit();}
 }
 stop(){this.generation++;this.bridge?.stop();this.bridge=undefined;this.current={status:'unavailable'};this.emit();}
}
declare global {interface Window {kortexHost:BrowserHost}}
if(!window.kortexHost){window.kortexHost=new BrowserHost();window.dispatchEvent(new Event('kortex:host'));}
