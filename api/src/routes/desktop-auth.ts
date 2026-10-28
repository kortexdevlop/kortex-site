import {z} from 'zod';
import type {Env} from '../env';
import {json,failure} from '../lib/http';
import {AdminError,readBounded} from '../lib/admin-error';
import {randomToken,opaqueHash,b64,encoder} from '../security/sessions';
import {userScope} from '../security/user-scope';
import {deviceSession,DEVICE_SCOPES} from '../security/device-session';
const opaque=z.string().regex(/^[A-Za-z0-9_-]{43}$/);
const startSchema=z.object({challenge:opaque,challenge_method:z.literal('S256'),state:opaque,redirect_uri:z.string().max(200)}).strict();
async function body(request:Request){if(request.headers.get('Content-Type')?.split(';')[0]!=='application/json')throw Error('JSON');return JSON.parse(new TextDecoder().decode(await readBounded(request.body,2048)))}
export function loopback(value:string){const u=new URL(value);if(u.protocol!=='http:'||u.hostname!=='127.0.0.1'||u.pathname!=='/callback'||u.search||u.hash||u.username||u.password||Number(u.port)<1024||Number(u.port)>65535)throw Error('LOOPBACK');return u.href}
export async function desktopAuth(request:Request,env:Env,id:string,viaWeb:boolean):Promise<Response|null>{
 const url=new URL(request.url),match=/^\/api\/v1\/auth\/desktop\/(start|request|approve|token|logout)$/.exec(url.pathname);if(!match)return null;const action=match[1];
 if(request.method!==(action==='request'?'GET':'POST'))return failure('BAD_REQUEST','Method not allowed',405,id);
 try{
  if(!env.SESSION_HMAC_SECRET||env.PASSWORD_AUTH_ENABLED!=='true'&&env.OIDC_ENABLED!=='true')return failure('UNAVAILABLE','Device sign-in unavailable',503,id);
  if(action==='request'||action==='approve'){
   if(!viaWeb)return failure('NOT_FOUND','Route not found',404,id);if(request.headers.has('Authorization'))return failure('FORBIDDEN','Browser session required',403,id);const owner=await userScope(request,env,true,action==='approve');
   if(action==='request'){
    if(url.searchParams.size!==1)throw Error('QUERY');const requestId=z.uuid().parse(url.searchParams.get('id')),row=await env.DB.prepare('SELECT id,redirect_uri,expires_at FROM device_authorizations WHERE id=? AND user_id IS NULL AND expires_at>?').bind(requestId,Date.now()).first();if(!row)return failure('NOT_FOUND','Sign-in request expired',404,id);return json({...row,scopes:DEVICE_SCOPES},200,id);
   }
   const input=z.object({request_id:z.uuid()}).strict().parse(await body(request)),code=randomToken();const row=await env.DB.prepare('UPDATE device_authorizations SET user_id=?,code_hash=? WHERE id=? AND user_id IS NULL AND expires_at>? RETURNING redirect_uri,state').bind(owner.user_id,await opaqueHash(env,'kortex-device-code-v1',code),input.request_id,Date.now()).first<{redirect_uri:string;state:string}>();if(!row)return failure('CONFLICT','Sign-in request expired or already approved',409,id);
   const redirect=new URL(loopback(row.redirect_uri));redirect.search=new URLSearchParams({code,state:row.state}).toString();return json({redirect_url:redirect.href},200,id);
  }
  if(viaWeb||request.headers.has('Origin'))return failure('FORBIDDEN','Native client required',403,id);if(url.searchParams.size)throw Error('QUERY');
  if(action==='logout'){const owner=await deviceSession(request,env,'device:logout');await env.DB.prepare('UPDATE device_sessions SET revoked_at=? WHERE id=? AND user_id=?').bind(Date.now(),owner.id,owner.user_id).run();return json(null,204,id)}
  if(action==='start'){
   const input=startSchema.parse(await body(request)),redirect=loopback(input.redirect_uri),now=Date.now();await env.DB.prepare('DELETE FROM device_authorizations WHERE expires_at<=?').bind(now).run();
   await env.DB.prepare('DELETE FROM ai_quota WHERE expires_at<=?').bind(now).run();const bucket='desktop-start-'+Math.floor(now/60000),count=await env.DB.prepare('INSERT INTO ai_quota(bucket,requests,reserved_tokens,expires_at) VALUES(?,1,0,?) ON CONFLICT(bucket) DO UPDATE SET requests=requests+1 RETURNING requests').bind(bucket,now+120000).first<{requests:number}>();if(!count||count.requests>60)return failure('RATE_LIMITED','Too many sign-in requests',429,id);
   const requestId=crypto.randomUUID(),result=await env.DB.prepare('INSERT INTO device_authorizations(id,challenge,state,redirect_uri,expires_at) SELECT ?,?,?,?,? WHERE (SELECT count(*) FROM device_authorizations)<1000').bind(requestId,input.challenge,input.state,redirect,now+600000).run();if(result.meta.changes!==1)return failure('RATE_LIMITED','Sign-in temporarily unavailable',429,id);
   return json({request_id:requestId,authorization_url:env.PUBLIC_WEB_ORIGIN+'/account/connect-desktop?request_id='+requestId,expires_at:now+600000},200,id);
  }
  const input=z.object({request_id:z.uuid(),code:opaque,verifier:opaque}).strict().parse(await body(request));const challenge=b64(new Uint8Array(await crypto.subtle.digest('SHA-256',encoder.encode(input.verifier))));
  const row=await env.DB.prepare('UPDATE device_authorizations SET consumed_at=? WHERE id=? AND code_hash=? AND challenge=? AND user_id IS NOT NULL AND consumed_at IS NULL AND expires_at>? RETURNING user_id').bind(Date.now(),input.request_id,await opaqueHash(env,'kortex-device-code-v1',input.code),challenge,Date.now()).first<{user_id:string}>();if(!row)return failure('UNAUTHORIZED','Invalid or consumed authorization code',401,id);
  const token='kd_'+randomToken(),deviceId=crypto.randomUUID(),now=Date.now();await env.DB.batch([env.DB.prepare('DELETE FROM device_sessions WHERE expires_at<=? OR revoked_at IS NOT NULL').bind(now),env.DB.prepare('INSERT INTO device_sessions(id,user_id,token_hash,created_at,expires_at) SELECT ?,?,?,?,? WHERE (SELECT count(*) FROM device_sessions WHERE user_id=?)<20').bind(deviceId,row.user_id,await opaqueHash(env,'kortex-device-token-v1',token),now,now+604800000,row.user_id)]);
  const stored=await env.DB.prepare('SELECT id FROM device_sessions WHERE id=?').bind(deviceId).first();if(!stored)return failure('RATE_LIMITED','Device limit reached',429,id);
  return json({access_token:token,token_type:'Bearer',device_id:deviceId,user_id:row.user_id,expires_at:now+604800000,scopes:DEVICE_SCOPES},200,id);
 }catch(error){if(error instanceof AdminError)return failure(error.code,error.message,error.status,id);return failure('BAD_REQUEST','Invalid device sign-in request',400,id)}
}
