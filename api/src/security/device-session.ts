import type {Env} from '../env';
import {opaqueHash} from './sessions';
import {SyncError} from '../services/sync';
export const DEVICE_SCOPES=['identity:read','sync:read','sync:write','device:logout'] as const;
export async function deviceSession(request:Request,env:Env,scope:string){
 const match=/^Bearer (kd_[A-Za-z0-9_-]{43})$/.exec(request.headers.get('Authorization')??'');if(!match||![...DEVICE_SCOPES,'ai:plan'].includes(scope))throw new SyncError(403,'FORBIDDEN','A scoped device session is required');
 const hash=await opaqueHash(env,'kortex-device-token-v1',match[1]),row=await env.DB.prepare('SELECT id,user_id,expires_at,ai_allowed FROM device_sessions WHERE token_hash=? AND revoked_at IS NULL AND expires_at>?').bind(hash,Date.now()).first<{id:string;user_id:string;expires_at:number;ai_allowed:number}>();if(!row)throw new SyncError(401,'UNAUTHORIZED','Device session expired or revoked');if(scope==='ai:plan'&&!row.ai_allowed)throw new SyncError(403,'FORBIDDEN','Enable AI for this device in Web account settings');
 if(request.headers.has('Origin'))throw new SyncError(403,'FORBIDDEN','Native device session required');return {...row,hash};
}
