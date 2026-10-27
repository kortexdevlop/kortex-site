import type {Env} from '../env';
import {failure,json} from '../lib/http';
import {BadQuery,CatalogUnavailable,cursorKey,slug} from '../lib/cursor';
import {CatalogIntegrityError,list,detail,query} from '../services/catalog';
async function cached(request:Request,value:unknown,id:string,origin?:string,link?:string) {
 const digest=await crypto.subtle.digest('SHA-256',new TextEncoder().encode(JSON.stringify(value)));
 const etag='"'+Array.from(new Uint8Array(digest),b=>b.toString(16).padStart(2,'0')).join('')+'"';
 const condition=request.headers.get('If-None-Match');
 const matched=condition?.split(',').some(t=>t.trim()==='*'||t.trim().replace(/^W\//,'')===etag);
 return json(value,matched?304:200,id,origin,request.method==='HEAD',{'Cache-Control':'public, max-age=0, must-revalidate',ETag:etag,'Access-Control-Expose-Headers':'ETag, X-Request-ID, Link',...(link?{Link:link}:{})});
}
export async function tools(request:Request,env:Env,id:string,origin?:string):Promise<Response|null> {
 const url=new URL(request.url),match=/^\/api\/v1\/tools(?:\/([^/]+)(\/actions)?)?$/.exec(url.pathname);
 if(!match)return null;const head=request.method==='HEAD';
 if(!['GET','HEAD'].includes(request.method))return failure('BAD_REQUEST','Method not allowed',405,id,origin,head,{Allow:'GET, HEAD, OPTIONS'});
 try {
  let value:unknown;
  if(!match[1]){query(url.searchParams);value=await list(env.DB,url.searchParams,await cursorKey(env.CURSOR_SIGNING_KEY))}
  else {
   if(!slug.safeParse(match[1]).success||url.searchParams.size)throw new BadQuery();
   value=await detail(env.DB,match[1],Boolean(match[2]));if(value===null)return failure('NOT_FOUND','Tool not found',404,id,origin,head);
  }
  const tool=(value as {tool?:{id:string;version:string}}).tool;
  return await cached(request,value,id,origin,tool?`</api/v1/downloads/${tool.id}/${encodeURIComponent(tool.version)}>; rel="download"`:undefined);
 }catch(error){
  if(error instanceof BadQuery)return failure('BAD_REQUEST','Invalid catalog query',400,id,origin,head);
  if(error instanceof CatalogIntegrityError)return failure('INTEGRITY_ERROR','Catalog integrity check failed',503,id,origin,head);
  if(error instanceof CatalogUnavailable)return failure('UNAVAILABLE','Catalog configuration unavailable',503,id,origin,head);
  return failure('UNAVAILABLE','Catalog unavailable',503,id,origin,head);
 }
}
