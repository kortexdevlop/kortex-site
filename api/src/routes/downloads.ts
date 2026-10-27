import type {Env} from '../env';
import {failure,json} from '../lib/http';
import {slug} from '../lib/cursor';
import {downloadRow} from '../services/database';
import {checkedHead,checkedBody,StorageIntegrityError} from '../services/storage';
export const semver=/^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;
export async function downloads(request:Request,env:Env,id:string,origin?:string):Promise<Response|null> {
 const url=new URL(request.url),match=/^\/api\/v1\/downloads\/([^/]+)\/([^/]+)$/.exec(url.pathname);if(!match)return null;
 try{match[1]=decodeURIComponent(match[1]);match[2]=decodeURIComponent(match[2])}catch{return failure('BAD_REQUEST','Invalid download query',400,id,origin,request.method==='HEAD')}
 const head=request.method==='HEAD';if(!['GET','HEAD'].includes(request.method))return failure('BAD_REQUEST','Method not allowed',405,id,origin,head,{Allow:'GET, HEAD, OPTIONS'});
 if(!slug.safeParse(match[1]).success||match[2].length>128||!semver.test(match[2])||url.searchParams.size)return failure('BAD_REQUEST','Invalid download query',400,id,origin,head);
 try {
  const artifact=await downloadRow(env.DB,match[1],match[2]);if(!artifact)return failure('NOT_FOUND','Artifact not found',404,id,origin,head);
  if(artifact.revoked)return failure('FORBIDDEN','Artifact revoked',410,id,origin,head);
  if(artifact.tool_status!=='published'||artifact.version_status!=='published'||!artifact.published)return failure('NOT_FOUND','Artifact not published',404,id,origin,head);
  if(!artifact.verified)throw new StorageIntegrityError();
  if(!await checkedHead(env.TOOLS_BUCKET,artifact))return failure('NOT_FOUND','Artifact not found',404,id,origin,head);
  const etag='"'+artifact.sha256+'"';
  // Revalidate against D1 on every request. A long public TTL would bypass revocation.
  const headers=new Headers(json(null,204,id,origin,false,{'Cache-Control':'public, max-age=0, must-revalidate',ETag:etag,'Access-Control-Expose-Headers':'ETag, X-Kortex-SHA256, X-Request-ID'}).headers);
  headers.set('Content-Type','application/vnd.kortex.tool+zip');headers.set('X-Kortex-SHA256',artifact.sha256);
  headers.set('Content-Disposition',`attachment; filename="${match[1]}-${match[2]}.tool"`);
  if(request.headers.get('If-None-Match')?.split(',').some(t=>t.trim()==='*'||t.trim().replace(/^W\//,'')===etag))return new Response(null,{status:304,headers});
  headers.set('Content-Length',String(artifact.byte_length));if(head)return new Response(null,{headers});
  const body=await checkedBody(env.TOOLS_BUCKET,artifact);if(!body)return failure('NOT_FOUND','Artifact not found',404,id,origin);
  return new Response(body,{headers});
 }catch(error){return failure(error instanceof StorageIntegrityError?'INTEGRITY_ERROR':'UNAVAILABLE','Artifact unavailable',503,id,origin,head)}
}
