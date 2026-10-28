/** Versioned, local window metadata. Tool state, files and credentials never enter this model. */
import type {WorkspaceBounds as Bounds,WorkspaceWindow,WorkspaceState} from '../../../shared/typescript/src/index';
export type {Bounds,WorkspaceWindow,WorkspaceState};
export const emptyWorkspace=():WorkspaceState=>({workspace_version:'1.0',windows:[],focused:null});
export function newWindow(tool_id:string,version:string,bounds:Bounds):WorkspaceWindow{return{window_id:crypto.randomUUID(),instance_id:crypto.randomUUID(),tool_id,version,bounds:structuredClone(bounds),mode:'normal',z:0}}
const exact=(v:unknown,keys:string[]):v is Record<string,unknown>=>!!v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join(',')===[...keys].sort().join(',');
const semver=new RegExp("^(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)\\.(0|[1-9][0-9]*)(?:-(?:(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*)(?:\\.(?:0|[1-9][0-9]*|[0-9]*[A-Za-z-][0-9A-Za-z-]*))*))?(?:\\+[0-9A-Za-z-]+(?:\\.[0-9A-Za-z-]+)*)?$");
const uuid=(s:unknown):s is string=>typeof s==='string'&&/^[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}$/.test(s);
function viewport(v:{width:number;height:number}){if(!Number.isFinite(v.width)||!Number.isFinite(v.height)||v.width<=0||v.height<=0)throw Error('VIEWPORT');return{width:Math.max(320,v.width),height:Math.max(240,v.height)}}
export function clampBounds(b:Bounds,v:{width:number;height:number}):Bounds{const p=viewport(v);if(!Object.values(b).every(Number.isFinite))throw Error('BOUNDS');const width=Math.max(320,Math.min(p.width,b.width)),height=Math.max(240,Math.min(p.height,b.height));return{x:Math.max(0,Math.min(p.width-width,b.x)),y:Math.max(0,Math.min(p.height-height,b.y)),width,height}}
export function restoreWorkspace(value:unknown,v:{width:number;height:number}):WorkspaceState{
 if(!exact(value,['workspace_version','windows','focused'])||value.workspace_version!=='1.0'||!Array.isArray(value.windows)||value.windows.length>8)throw Error('WORKSPACE');
 const ids=new Set<string>(),instances=new Set<string>();const windows=value.windows.map((w:unknown)=>{if(!exact(w,['window_id','instance_id','tool_id','version','bounds','mode','z'])||!uuid(w.window_id)||!uuid(w.instance_id)||ids.has(w.window_id)||instances.has(w.instance_id)||typeof w.tool_id!=='string'||w.tool_id.length<3||w.tool_id.length>64||!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(w.tool_id)||typeof w.version!=='string'||w.version.length>100||!semver.test(w.version)||!['normal','minimized','maximized'].includes(w.mode as string)||!Number.isSafeInteger(w.z)||Number(w.z)<0||!exact(w.bounds,['x','y','width','height'])||!Object.values(w.bounds).every(n=>typeof n==='number'&&Number.isFinite(n)))throw Error('WINDOW');ids.add(w.window_id);instances.add(w.instance_id);return{...w,bounds:clampBounds(w.bounds as unknown as Bounds,v)} as unknown as WorkspaceWindow}).sort((a,b)=>a.z-b.z).map((w,z)=>({...w,z}));
 if(value.focused!==null&&(!uuid(value.focused)||!windows.some(w=>w.window_id===value.focused&&w.mode!=='minimized')))throw Error('FOCUS');return{workspace_version:'1.0',windows,focused:value.focused as string|null};
}
export type WorkspaceCommand={kind:'open';window:WorkspaceWindow}|{kind:'close'|'focus'|'minimize'|'maximize'|'restore';id:string}|{kind:'move';id:string;bounds:Bounds};
export function transition(state:WorkspaceState,command:WorkspaceCommand,v:{width:number;height:number}):WorkspaceState{
 const s=restoreWorkspace(state,v);let focus=s.focused;
 if(command.kind==='open'){if(s.windows.some(w=>w.instance_id===command.window.instance_id))throw Error('DUPLICATE_INSTANCE');if(s.windows.length===8)throw Error('WINDOW_LIMIT');s.windows.push({...command.window,z:s.windows.length});focus=command.window.mode==='minimized'?focus:command.window.window_id}
 else{const w=s.windows.find(w=>w.window_id===command.id);if(!w)throw Error('UNKNOWN_WINDOW');switch(command.kind){case'close':s.windows=s.windows.filter(x=>x!==w);break;case'minimize':w.mode='minimized';break;case'maximize':w.mode='maximized';focus=w.window_id;break;case'restore':w.mode='normal';focus=w.window_id;break;case'focus':if(w.mode==='minimized')w.mode='normal';focus=w.window_id;break;case'move':w.bounds=clampBounds(command.bounds,v);break;}}
 if(!s.windows.some(w=>w.window_id===focus&&w.mode!=='minimized'))focus=s.windows.filter(w=>w.mode!=='minimized').at(-1)?.window_id??null;
 s.windows=s.windows.filter(w=>w.window_id!==focus).concat(s.windows.filter(w=>w.window_id===focus)).map((w,z)=>({...w,z}));s.focused=focus;return restoreWorkspace(s,v);
}
export const workspaceStorageKey='kortex.workspace.v1';
export function saveWorkspace(storage:Pick<Storage,'setItem'>,state:WorkspaceState,v:{width:number;height:number}){storage.setItem(workspaceStorageKey,JSON.stringify(restoreWorkspace(state,v)))}
export function loadWorkspace(storage:Pick<Storage,'getItem'>,v:{width:number;height:number}):WorkspaceState{try{const raw=storage.getItem(workspaceStorageKey);return raw&&raw.length<=32768?restoreWorkspace(JSON.parse(raw),v):emptyWorkspace()}catch{return emptyWorkspace()}}
