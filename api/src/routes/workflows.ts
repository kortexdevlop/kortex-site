import type {Env} from '../env';
import {json,failure} from '../lib/http';
import {AdminError,readBounded} from '../lib/admin-error';
import {userScope} from '../security/user-scope';
import {ensureSync,SyncError} from '../services/sync';
import {persistentWorkflow} from '../../../website/src/lib/workflow-storage';
const route=/^\/api\/v1\/me\/workflows(?:\/([a-zA-Z0-9_-]{1,64}))?$/;
export async function workflows(request:Request,env:Env,id:string,viaWeb:boolean):Promise<Response|null>{
 const url=new URL(request.url),match=route.exec(url.pathname);if(!match)return null;let key=match[1];const write=request.method!=='GET';
 if(!['GET',...(key?['PUT','DELETE']:['POST'])].includes(request.method))return failure('BAD_REQUEST','Method not allowed',405,id,undefined,false,{Allow:key?'GET, PUT, DELETE':'GET, POST'});
 try{
  if(url.searchParams.size)throw new SyncError(400,'BAD_REQUEST','Selectors are not accepted');const owner=await userScope(request,env,viaWeb,write);await ensureSync(env.DB,owner.user_id);
  const state=await env.DB.prepare('SELECT s.workflows_revision AS revision,u.sync_enabled FROM user_sync s JOIN users u ON s.user_id=u.id WHERE s.user_id=?').bind(owner.user_id).first<{revision:number;sync_enabled:number}>();if(!state)throw new SyncError(401,'UNAUTHORIZED','Account unavailable');if(!state.sync_enabled)throw new SyncError(403,'FORBIDDEN','Synchronization is disabled');
  let revision=state.revision;
  if(write){
   const expected=/^"workflows-(0|[1-9][0-9]{0,15})"$/.exec(request.headers.get('If-Match')??'');if(!expected||Number(expected[1])!==revision||!Number.isSafeInteger(revision)||revision>=9007199254740990)throw new SyncError(409,'CONFLICT','Refresh workflows before writing');
   let definition:string|null=null;
   if(['PUT','POST'].includes(request.method)){
    if(request.headers.get('Content-Type')?.split(';')[0].trim()!=='application/json')throw new SyncError(415,'BAD_REQUEST','JSON required');
    const raw=await readBounded(request.body,262144);try{const value=persistentWorkflow(JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(raw)));if(request.method==='POST')key=value.id;else if(value.id!==key)throw Error();definition=JSON.stringify(value)}catch{throw new SyncError(400,'BAD_REQUEST','Invalid workflow or private file value')}
   }else if(request.body)throw new SyncError(400,'BAD_REQUEST','No body expected');
   if(request.method==='POST'&&await env.DB.prepare('SELECT workflow_id FROM user_workflows WHERE user_id=? AND workflow_id=?').bind(owner.user_id,key).first())throw new SyncError(409,'CONFLICT','Workflow already exists');
   const mutation=crypto.randomUUID(),now=Date.now(),results=await env.DB.batch([
    env.DB.prepare(`UPDATE user_sync SET workflows_revision=workflows_revision+1,workflows_mutation=? WHERE user_id=? AND workflows_revision=? AND EXISTS(SELECT 1 FROM users WHERE id=? AND sync_enabled=1)
     AND (EXISTS(SELECT 1 FROM user_workflows WHERE user_id=? AND workflow_id=?) OR (SELECT count(*) FROM user_workflows WHERE user_id=?)<100)
     AND (SELECT coalesce(sum(length(CAST(definition_json AS BLOB))),0) FROM user_workflows WHERE user_id=? AND workflow_id!=?)+length(CAST(coalesce(?,'') AS BLOB))<=524288`).bind(mutation,owner.user_id,revision,owner.user_id,owner.user_id,key,owner.user_id,owner.user_id,key,definition),
    env.DB.prepare(`INSERT INTO user_workflows SELECT user_id,?,?,workflows_revision,? FROM user_sync WHERE user_id=? AND workflows_mutation=? ON CONFLICT(user_id,workflow_id) DO UPDATE SET definition_json=excluded.definition_json,revision=excluded.revision,updated_at=excluded.updated_at`).bind(key,definition,now,owner.user_id,mutation)
   ]);if(results[0].meta.changes!==1)throw new SyncError(409,'CONFLICT','Workflows changed or storage quota reached');revision++;
  }
  const rows=await env.DB.prepare('SELECT workflow_id,definition_json,revision,updated_at FROM user_workflows WHERE user_id=? ORDER BY workflow_id LIMIT 100').bind(owner.user_id).all<{workflow_id:string;definition_json:string|null;revision:number;updated_at:number}>();const items=rows.results.map(r=>({id:r.workflow_id,workflow:r.definition_json===null?null:JSON.parse(r.definition_json),revision:r.revision,updated_at:r.updated_at}));
  const item=key?items.find(x=>x.id===key):undefined;if(key&&!write&&(!item||item.workflow===null))throw new SyncError(404,'NOT_FOUND','Workflow not found');
  return json(key?{revision,item}:{revision,items},200,id,undefined,false,{ETag:`"workflows-${revision}"`,'Cache-Control':'private, no-store, no-transform'});
 }catch(e){if(e instanceof AdminError)return failure(e.code,e.message,e.status,id);return failure('UNAVAILABLE','Workflow storage unavailable',503,id)}
}
