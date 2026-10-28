import type {Env} from '../env';
import {session} from './sessions';
import {checkCsrf} from './csrf';
import {SyncError} from '../services/sync';
export async function userScope(request:Request,env:Env,viaWeb:boolean,write:boolean){
 if(request.headers.has('Authorization'))throw new SyncError(403,'FORBIDDEN','A scoped account session is required');
 if(!viaWeb)throw new SyncError(404,'NOT_FOUND','Route not found');const current=await session(request,env);if(!current)throw new SyncError(401,'UNAUTHORIZED','Account session required');
 if(request.headers.has('X-Kortex-Account')&&request.headers.get('X-Kortex-Account')!==current.user_id)throw new SyncError(403,'FORBIDDEN','The account changed; sign in again');
 if(write&&!await checkCsrf(request,env,current.token))throw new SyncError(403,'FORBIDDEN','Invalid request verification');return current;
}
