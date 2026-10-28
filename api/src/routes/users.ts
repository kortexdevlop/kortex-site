import {z} from 'zod';
import type {Env} from '../env';
import {json,failure} from '../lib/http';
import {AdminError,readBounded} from '../lib/admin-error';
import {userScope} from '../security/user-scope';
import {opaqueHash,cookie,setCookie,SESSION_COOKIE,FLOW_COOKIE} from '../security/sessions';
import {etag,revision,readLibrary,readPreferences,mutateLibrary,mutatePreferences,librarySchema,settingsSchema,SyncError,ensureSync} from '../services/sync';
const route=/^\/api\/v1\/me\/(library|preferences|export|data|devices)$/;
async function body(request:Request){if(request.headers.get('Content-Type')?.split(';')[0].trim()!=='application/json')throw new SyncError(415,'BAD_REQUEST','JSON required');return JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(await readBounded(request.body,16384)))}
export async function users(request:Request,env:Env,id:string,viaWeb:boolean):Promise<Response|null>{
 const url=new URL(request.url),match=route.exec(url.pathname);if(!match)return null;const name=match[1],methods=name==='library'?['GET','PUT','DELETE']:name==='preferences'?['GET','PATCH']:name==='export'?['GET']:name==='devices'?['GET','DELETE']:['DELETE'];
 if(!methods.includes(request.method))return failure('BAD_REQUEST','Method not allowed',405,id,undefined,false,{Allow:methods.join(', ')});
 try{
  if(url.searchParams.size)throw new SyncError(400,'BAD_REQUEST','Account selectors are not accepted');
  const owner=await userScope(request,env,viaWeb,request.method!=='GET',['export','data','devices'].includes(name)?'account:'+name:request.method==='GET'?'sync:read':'sync:write');
  if(name==='devices'){
   if(request.method==='DELETE'){const input=z.object({id:z.uuid()}).strict().parse(await body(request));await env.DB.prepare('UPDATE device_sessions SET revoked_at=? WHERE id=? AND user_id=?').bind(Date.now(),input.id,owner.user_id).run();return json(null,204,id)}
   const rows=await env.DB.prepare('SELECT id,label,created_at,expires_at,revoked_at FROM device_sessions WHERE user_id=? ORDER BY created_at DESC LIMIT 20').bind(owner.user_id).all();return json({devices:rows.results},200,id);
  }
  if(name==='preferences'){
   const data=request.method==='GET'?await readPreferences(env.DB,owner.user_id):await mutatePreferences(env.DB,owner.user_id,revision(request.headers.get('If-Match'),'preferences'),settingsSchema.parse(await body(request)));
   return json(data,200,id,undefined,false,{ETag:etag('preferences',data.revision)});
  }
  if(name==='library'){
   let data=await readLibrary(env.DB,owner.user_id);if(!data.sync_enabled)throw new SyncError(403,'FORBIDDEN','Synchronization is disabled');
   if(request.method!=='GET'){
    const expected=revision(request.headers.get('If-Match'),'library');if(request.method==='DELETE'){
     if(request.body)throw new SyncError(400,'BAD_REQUEST','No body expected');if(expected!==data.revision)throw new SyncError(409,'CONFLICT','Library changed; refresh before retrying');
     const removals=data.items.flatMap(item=>item.tags.map(tag=>({tool_id:item.tool_id,tag})));if(removals.length>100)throw new SyncError(413,'BAD_REQUEST','Remove at most 100 tags per request');
     if(removals.length)data=await mutateLibrary(env.DB,owner.user_id,expected,{adds:[],removes:removals.map(r=>({tool_id:r.tool_id,tags:[r.tag]}))});
    }else data=await mutateLibrary(env.DB,owner.user_id,expected,librarySchema.parse(await body(request)));
   }
   const {records,...publicData}=data;return json(publicData,200,id,undefined,false,{ETag:etag('library',data.revision)});
  }
  await ensureSync(env.DB,owner.user_id);
  if(name==='export'){
   const rows=await env.DB.batch([env.DB.prepare('SELECT u.id,u.created_at,u.sync_enabled,s.library_revision,s.preferences_revision FROM users u JOIN user_sync s ON s.user_id=u.id WHERE u.id=?').bind(owner.user_id),env.DB.prepare('SELECT email,email_verified,created_at FROM password_credentials WHERE user_id=?').bind(owner.user_id),env.DB.prepare('SELECT issuer,subject,created_at FROM external_identities WHERE user_id=?').bind(owner.user_id),env.DB.prepare('SELECT tool_id,tag,added_revision,removed_revision FROM user_library WHERE user_id=? ORDER BY tool_id,tag').bind(owner.user_id),env.DB.prepare('SELECT data_json FROM user_preferences WHERE user_id=?').bind(owner.user_id),env.DB.prepare('SELECT entity,revision,created_at FROM sync_events WHERE user_id=? ORDER BY entity,revision').bind(owner.user_id),env.DB.prepare('SELECT created_at,expires_at,revoked_at FROM sessions WHERE user_id=? ORDER BY created_at').bind(owner.user_id)]);
   const user=rows[0].results[0] as {library_revision:number;preferences_revision:number}|undefined,preference=rows[4].results[0] as {data_json:string}|undefined;if(!user||!preference)throw new SyncError(401,'UNAUTHORIZED','Session unavailable');const tag=`"account-${user.library_revision}-${user.preferences_revision}"`;
   const devices=await env.DB.prepare('SELECT id,label,created_at,expires_at,revoked_at FROM device_sessions WHERE user_id=? ORDER BY created_at').bind(owner.user_id).all();
   const data={schema_version:1,exported_at:Date.now(),user,credentials:rows[1].results,identities:rows[2].results,library:rows[3].results,preferences:JSON.parse(preference.data_json),events:rows[5].results,sessions:rows[6].results,devices:devices.results};if(new TextEncoder().encode(JSON.stringify(data)).length>1048576)throw new SyncError(413,'BAD_REQUEST','Export exceeds the response quota');
   return json(data,200,id,undefined,false,{ETag:tag,'Content-Disposition':'attachment; filename="kortex-account.json"'});
  }
  z.object({confirm:z.literal('delete-account')}).strict().parse(await body(request));
  const expected=/^"account-(0|[1-9][0-9]{0,15})-(0|[1-9][0-9]{0,15})"$/.exec(request.headers.get('If-Match')??'');if(!expected||!expected.slice(1).every(v=>Number.isSafeInteger(Number(v))))throw new SyncError(409,'CONFLICT','A current account revision is required');
  const credentials=await env.DB.prepare('SELECT email_key FROM password_credentials WHERE user_id=?').bind(owner.user_id).first<{email_key:string}>();
  const deleted=await env.DB.prepare('DELETE FROM users WHERE id=? AND EXISTS(SELECT 1 FROM user_sync WHERE user_id=? AND library_revision=? AND preferences_revision=?) RETURNING id').bind(owner.user_id,owner.user_id,Number(expected[1]),Number(expected[2])).first<{id:string}>();if(!deleted)throw new SyncError(409,'CONFLICT','Account changed; refresh before retrying');
  // Purge only this account's short-lived password counters; shared anti-abuse counters remain.
  await env.DB.prepare('DELETE FROM ai_quota WHERE bucket=?').bind(await opaqueHash(env,'kortex-ai-quota-v1',owner.user_id+':'+Math.floor(Date.now()/86400000))).run();
  if(credentials){for(const [kind,window]of [['login',900000],['register',3600000]] as const){const bucket=await opaqueHash(env,'kortex-password-rate-v1',kind+':'+credentials.email_key+':'+Math.floor(Date.now()/window));await env.DB.prepare('DELETE FROM password_auth_limits WHERE bucket=?').bind(bucket).run()}}
  const browser=cookie(request,'__Host-kortex_password');if(browser)await env.DB.prepare('DELETE FROM password_auth_forms WHERE token_hash=?').bind(await opaqueHash(env,'kortex-password-form-v1',browser)).run();
  const flow=cookie(request,FLOW_COOKIE);if(flow)await env.DB.prepare('DELETE FROM oidc_flows WHERE browser_hash=?').bind(await opaqueHash(env,'kortex-browser-v1',flow)).run();
  const headers=new Headers();for(const key of [SESSION_COOKIE,FLOW_COOKIE,'__Host-kortex_password'])headers.append('Set-Cookie',setCookie(key,'',0));return json(null,204,id,undefined,false,headers);
 }catch(error){
  if(error instanceof AdminError)return failure(error.code,error.message,error.status,id);
  if(error instanceof z.ZodError||error instanceof SyntaxError)return failure('BAD_REQUEST','Invalid account data',400,id);
  return failure('UNAVAILABLE','Account service unavailable',503,id);
 }
}
