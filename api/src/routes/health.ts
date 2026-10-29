import {json} from '../lib/http';
export function health(request:Request,id:string,origin?:string,revision?:string):Response {
 // Liveness only. Never queries the database/bucket or serializes bindings/configuration.
 return json({status:'ok',api_version:'1'},200,id,origin,request.method==='HEAD',revision&&/^[a-f0-9]{40}$/.test(revision)?{'X-Kortex-Api-Revision':revision}:{});
}
