import {z} from 'zod';
import type {Env} from '../env';
import {json,failure,uuid} from '../lib/http';
import {slug} from '../lib/cursor';
import {AdminError,readBounded} from '../lib/admin-error';
import {authorize} from '../security/admin-auth';
import {publish,revoke,rollback} from '../services/publication';
import {semver} from './downloads';
const version=z.string().max(128).regex(semver),requestId=z.string().regex(uuid).transform(s=>s.toLowerCase());
const rollbackBody=z.object({request_id:requestId,version,expected_current_version:version}).strict();
const revokeBody=z.object({request_id:requestId,reason:z.string().trim().min(1).max(1000)}).strict();
export async function admin(request:Request,env:Env,id:string,origin?:string):Promise<Response|null> {
 const path=new URL(request.url).pathname;if(!path.startsWith('/api/v1/admin/'))return null;
 const rollbackPath=/^\/api\/v1\/admin\/tools\/([a-z0-9-]+)\/rollback$/.exec(path),revokePath=/^\/api\/v1\/admin\/artifacts\/([a-f0-9]{64})\/revoke$/.exec(path);
 const action=path==='/api/v1/admin/publications'?'publish':rollbackPath?'rollback':revokePath?'revoke':null;
 if(!action)return failure('NOT_FOUND','Admin route not found',404,id,origin);
 if(request.method!=='POST')return failure('BAD_REQUEST','Method not allowed',405,id,origin,request.method==='HEAD',{Allow:'POST'});
 try {
  const actor=await authorize(request,env,action);
  if(new URL(request.url).searchParams.size||request.headers.get('content-type')?.split(';')[0].trim()!=='application/json')throw new AdminError(400,'BAD_REQUEST','JSON request required');
  if(Number(request.headers.get('content-length'))>262144)throw new AdminError(413,'BAD_REQUEST','Request body too large');
  let body:unknown;try{body=JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(await readBounded(request.body,262144)))}catch(error){if(error instanceof AdminError)throw error;throw new AdminError(400,'BAD_REQUEST','Invalid JSON request')}
  let result:unknown;
  if(action==='publish')result=await publish(env,body,actor);
  else if(action==='rollback'){const parsed=rollbackBody.safeParse(body);if(!parsed.success||!slug.safeParse(rollbackPath![1]).success)throw new AdminError(400,'BAD_REQUEST','Invalid rollback request');result=await rollback(env,rollbackPath![1],parsed.data,actor)}
  else {const parsed=revokeBody.safeParse(body);if(!parsed.success)throw new AdminError(400,'BAD_REQUEST','Invalid revocation request');result=await revoke(env,revokePath![1],parsed.data,actor)}
  return json(result,200,id,origin);
 }catch(error){if(error instanceof AdminError)return failure(error.code,error.message,error.status,id,origin);return failure('UNAVAILABLE','Admin service unavailable',503,id,origin)}
}
