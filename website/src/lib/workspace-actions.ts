import {runtimeSession,type RuntimeStatus} from './runtime-session.ts';
import {isToolActions} from './tools.ts';
import {matchesValueSchema} from '../../../shared/typescript/src/value-schema.ts';
import type {Action} from '../../../shared/typescript/src/index';
import type {WorkspaceWindow} from './workspace-state';
export class WorkspaceError extends Error{code:string;constructor(code:string){super(code);this.code=code}}
export type InstanceView={instance_id:string;window_id:string;tool_id:string;version:string;status:RuntimeStatus;actions:Action[];last_result?:unknown};
type InstanceRecord=InstanceView&{session:ReturnType<typeof runtimeSession>;abort:AbortController;epoch:symbol;busy:boolean};
export type HistoryRow={at:number;tool_id:string;action_id:string;status:'completed'|'failed'|'cancelled'};
export const historyKey='kortex.workspace.history.v1';
export class WorkspaceActions{
 private records=new Map<string,InstanceRecord>();private retired=new Set<string>();private listeners=new Set<()=>void>();private history:HistoryRow[]=[];
 private origin:string;constructor(origin:string){this.origin=origin}
 subscribe(fn:()=>void){this.listeners.add(fn);return()=>{this.listeners.delete(fn)}}
 private emit(){for(const fn of this.listeners)fn()}
 views():InstanceView[]{return[...this.records.values()].map(r=>({instance_id:r.instance_id,window_id:r.window_id,tool_id:r.tool_id,version:r.version,status:r.status,actions:structuredClone(r.actions),last_result:r.last_result}))}
 get(id:string){const r=this.records.get(id);if(!r)throw new WorkspaceError('NOT_FOUND');return r}
 entries(){return this.history.map(r=>({...r}))}
 loadHistory(storage:Pick<Storage,'getItem'>){try{const raw=storage.getItem(historyKey);if(!raw||raw.length>65536)return;const rows=JSON.parse(raw);if(Array.isArray(rows)&&rows.length<=100)this.history=rows.filter(r=>r&&Object.keys(r).sort().join(',')==='action_id,at,status,tool_id'&&Number.isFinite(r.at)&&typeof r.tool_id==='string'&&/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(r.tool_id)&&r.tool_id.length<=64&&typeof r.action_id==='string'&&/^[a-z][a-z0-9_-]{0,63}$/.test(r.action_id)&&['completed','failed','cancelled'].includes(r.status));}catch{}this.emit()}
 private log(r:InstanceRecord,action_id:string,status:HistoryRow['status']){this.history=[...this.history.slice(-99),{at:Date.now(),tool_id:r.tool_id,action_id,status}];try{localStorage.setItem(historyKey,JSON.stringify(this.history))}catch{}this.emit()}
 async catalog(tool:string,signal:AbortSignal){const response=await fetch('/api/v1/tools/'+encodeURIComponent(tool)+'/actions',{credentials:'omit',cache:'no-store',redirect:'error',signal:AbortSignal.any([signal,AbortSignal.timeout(15000)])});if(!response.ok)throw new WorkspaceError('UNAVAILABLE');const value=await response.json();if(!isToolActions(value)||value.tool_id!==tool)throw new WorkspaceError('UNAVAILABLE');return value}
 attach(w:WorkspaceWindow,frame:HTMLIFrameElement){if(this.records.has(w.instance_id)||this.retired.has(w.instance_id))throw new WorkspaceError('DUPLICATE');const abort=new AbortController(),epoch=Symbol();let record:InstanceRecord;
  const session=runtimeSession(frame,this.origin,w.tool_id,w.version,status=>{if(record&&this.records.get(w.instance_id)?.epoch===epoch){record.status=status==='ready'&&!record.actions.length?'loading':status;this.emit()}});
  record={...w,status:'loading',actions:[],session,abort,epoch,busy:false};this.records.set(w.instance_id,record);this.emit();
  void Promise.all([session.ready,this.catalog(w.tool_id,abort.signal)]).then(([,data])=>{if(this.records.get(w.instance_id)?.epoch!==epoch)return;if(data.version!==w.version)throw new WorkspaceError('STALE');record.actions=structuredClone(data.actions);record.status='ready';this.emit()},()=>{if(this.records.get(w.instance_id)?.epoch===epoch){session.close();record.status='error';this.emit()}}).catch(()=>{if(this.records.get(w.instance_id)?.epoch===epoch){session.close();record.status='error';this.emit()}});
  return()=>{if(this.records.get(w.instance_id)?.epoch===epoch)this.close(w.instance_id)};
 }
 close(id:string){const r=this.records.get(id);if(!r)return;this.records.delete(id);this.retired.add(id);r.abort.abort();r.session.close();this.emit()}
 shutdown(){for(const id of [...this.records.keys()])this.close(id)}
 async dispatch(instance_id:string,action_ref:string,payload:unknown,signal=new AbortController().signal):Promise<unknown>{
  const r=this.get(instance_id);if(!action_ref.startsWith(r.tool_id+'.'))throw new WorkspaceError('FORBIDDEN');if(r.busy)throw new WorkspaceError('BUSY');if(r.status!=='ready')throw new WorkspaceError('UNAVAILABLE');const action=r.actions.find(a=>r.tool_id+'.'+a.id===action_ref);if(!action)throw new WorkspaceError('FORBIDDEN');
  if(new TextEncoder().encode(JSON.stringify(payload)).length>1048576||!matchesValueSchema(action.input_schema as unknown as Record<string,unknown>,payload))throw new WorkspaceError('INVALID_INPUT');
  const operation=AbortSignal.any([signal,r.abort.signal]);r.busy=true;try{if(navigator.onLine){const current=await this.catalog(r.tool_id,operation);if(current.version!==r.version||JSON.stringify(current.actions)!==JSON.stringify(r.actions))throw new WorkspaceError('STALE')}
   if(this.records.get(instance_id)?.epoch!==r.epoch)throw new WorkspaceError('NOT_FOUND');operation.throwIfAborted();const result=await r.session.invoke(action.id,structuredClone(payload),operation);if(this.records.get(instance_id)?.epoch!==r.epoch)throw new WorkspaceError('NOT_FOUND');
   if(!matchesValueSchema(action.output_schema as unknown as Record<string,unknown>,result))throw new WorkspaceError('INVALID_OUTPUT');r.last_result=structuredClone(result);this.log(r,action.id,'completed');return result;
  }catch(error){const code=!this.records.has(instance_id)?'NOT_FOUND':signal.aborted?'CANCELLED':error instanceof WorkspaceError?error.code:error instanceof Error&&['PERMISSION','CANCELLED','TIMEOUT','LIMIT','BUSY'].includes(error.message)?error.message:'UNAVAILABLE';this.log(r,action.id,code==='CANCELLED'||code==='NOT_FOUND'?'cancelled':'failed');throw new WorkspaceError(code)}finally{r.busy=false;this.emit()}
 }
}
