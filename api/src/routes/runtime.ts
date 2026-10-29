import type {Env} from '../env';
import {json,failure} from '../lib/http';
import {verifyRuntime,type RuntimeEnvelope} from '../../../shared/typescript/src/runtime';
export async function runtime(request:Request,env:Env,id:string,origin?:string):Promise<Response|null>{
 const url=new URL(request.url);if(url.pathname!=='/api/v1/runtime/channels')return null;if(!['GET','HEAD'].includes(request.method)||url.search)return failure('BAD_REQUEST','Unsupported runtime request',400,id,origin,request.method==='HEAD');
 try{if(!env.RUNTIME_CHANNELS||env.RUNTIME_CHANNELS.length>65536||!env.RUNTIME_ROOT)throw Error();const envelope=JSON.parse(env.RUNTIME_CHANNELS) as RuntimeEnvelope;await verifyRuntime(envelope,JSON.parse(env.RUNTIME_ROOT));const key=await env.DB.prepare('SELECT public_key_base64,revoked,not_before,expires FROM signing_keys WHERE id=?').bind(envelope.document.signing_key_id).first<{public_key_base64:string;revoked:number;not_before:number;expires:number}>();const cert=envelope.keys.keys.find(k=>k.id===envelope.document.signing_key_id);if(!key||key.revoked||key.public_key_base64!==cert?.public_key_base64||Date.now()<key.not_before||Date.now()>=key.expires)throw Error();return json(envelope,200,id,origin,request.method==='HEAD',{'Cache-Control':'no-store'})}catch{return failure('UNAVAILABLE','Signed runtime channels unavailable',503,id,origin,request.method==='HEAD')}
}
