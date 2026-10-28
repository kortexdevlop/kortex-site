import type {Env} from '../env';
import {session} from './sessions';
import {checkCsrf} from './csrf';
import {SyncError} from '../services/sync';
import {deviceSession} from './device-session';
export async function userScope(request:Request,env:Env,viaWeb:boolean,write:boolean,scope=write?'sync:write':'sync:read'){
 if(request.headers.has('Authorization')){const device=await deviceSession(request,env,scope);if(request.headers.has('X-Kortex-Account')&&request.headers.get('X-Kortex-Account')!==device.user_id)throw new SyncError(403,'FORBIDDEN','The account changed');return {user_id:device.user_id,token:'',hash:device.hash,expires_at:device.expires_at}}
 if(!viaWeb)throw new SyncError(404,'NOT_FOUND','Route not found');const current=await session(request,env);if(!current)throw new SyncError(401,'UNAUTHORIZED','Account session required');
 if(request.headers.has('X-Kortex-Account')&&request.headers.get('X-Kortex-Account')!==current.user_id)throw new SyncError(403,'FORBIDDEN','The account changed; sign in again');
 if(write&&!await checkCsrf(request,env,current.token))throw new SyncError(403,'FORBIDDEN','Invalid request verification');return current;
}
