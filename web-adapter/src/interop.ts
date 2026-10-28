import './host';
import {EmbedSession} from './iframe';
import {TrustStore} from './trust';
export function subscribe(callback:(value:unknown)=>void){const send=()=>callback(window.kortexHost.snapshot());window.addEventListener('kortex:host',send);window.addEventListener('online',send);window.addEventListener('offline',send);send();return ()=>{window.removeEventListener('kortex:host',send);window.removeEventListener('online',send);window.removeEventListener('offline',send)}}
export function command(value:Record<string,unknown>){void window.kortexHost.command(value)}
export function mountEmbed(){const config=(window as any).kortexEmbedConfig;if(!config||window.parent===window)return;
 const trust=new TrustStore(config.keys);const session=new EmbedSession({parentOrigin:config.parent_origin,clientVersion:'1.0'},async(id,version)=>{
  const descriptor=config.artifacts.find((a:any)=>a.tool_id===id&&a.version===version);if(!descriptor)throw Error('UNKNOWN_TOOL');await window.kortexHost.loadTool(descriptor,trust,{api_origin:config.api_origin,allowed_parent_origin:config.parent_origin,tool_id:id,tool_version:version});return {tool_id:id,version,status:'ready'};
 },()=>{},async data=>{if(data.kind==='AI_REVIEW'){window.kortexHost.reviewAi(data.plan);return {reviewed:true}}if(data.kind==='AI_DECIDE'&&Number.isSafeInteger(data.index)&&typeof data.approved==='boolean')return window.kortexHost.decideAi(data.index as number,data.approved);throw Error('INVALID_AI_COMMAND')});session.listen();}
mountEmbed();
export {VerifiedTool,WebBridge,startVerifiedTool,ToolLoader,TrustStore} from './host';
