import {WorkerEntrypoint} from 'cloudflare:workers';
import {sitemap} from './routes/sitemap';
import {validateEnv,type Env} from './env';
import {failure,json,requestId} from './lib/http';
import {health} from './routes/health';
import {tools} from './routes/tools';
import {downloads} from './routes/downloads';
import {admin} from './routes/admin';
import {auth} from './routes/auth';
import {passwordAuth} from './routes/password-auth';
import {users} from './routes/users';
import {desktopAuth} from './routes/desktop-auth';
import {ai} from './routes/ai';
import {workflows} from './routes/workflows';
export async function handle(request:Request,bindings:unknown,viaWeb=false):Promise<Response> {
 const id=requestId(request),head=request.method==='HEAD';
 try {
  const env=validateEnv(bindings);if(!env)return failure('UNAVAILABLE','Service configuration unavailable',503,id,undefined,head);
  const sentOrigin=request.headers.get('Origin');
  if(sentOrigin&&sentOrigin!==env.PUBLIC_WEB_ORIGIN)return failure('FORBIDDEN','Origin not allowed',403,id,undefined,head);
  const origin=sentOrigin===env.PUBLIC_WEB_ORIGIN?env.PUBLIC_WEB_ORIGIN:undefined;
  if(request.method==='OPTIONS'){
   const method=request.headers.get('Access-Control-Request-Method');
   const headers=(request.headers.get('Access-Control-Request-Headers')??'').split(',').map(h=>h.trim().toLowerCase()).filter(Boolean);
   if(!origin||!method||!['GET','HEAD'].includes(method)||headers.some(h=>!['content-type','x-request-id','if-none-match'].includes(h)))return failure('BAD_REQUEST','Unsupported preflight',400,id,origin);
   return json(null,204,id,origin,false,{'Access-Control-Allow-Methods':'GET, HEAD, OPTIONS','Access-Control-Allow-Headers':'Content-Type, X-Request-ID, If-None-Match'});
  }
  if(new URL(request.url).pathname==='/api/v1/health'){
   if(!['GET','HEAD'].includes(request.method))return failure('BAD_REQUEST','Method not allowed',405,id,origin,head,{Allow:'GET, HEAD, OPTIONS'});
   return health(request,id,origin);
  }
  const device=await desktopAuth(request,env,id,viaWeb);if(device)return device;
  const proposal=await ai(request,env,id,viaWeb);if(proposal)return proposal;
  const password=await passwordAuth(request,env,id,viaWeb);if(password)return password;
  const workflow=await workflows(request,env,id,viaWeb);if(workflow)return workflow;
  const account=await users(request,env,id,viaWeb);if(account)return account;
  const authentication=await auth(request,env,id,viaWeb);if(authentication)return authentication;
  const administration=await admin(request,env,id,origin);if(administration)return administration;
  const catalog=await tools(request,env,id,origin);if(catalog)return catalog;
  const download=await downloads(request,env,id,origin);if(download)return download;
  return failure('NOT_FOUND','Route not found',404,id,origin,head);
 }catch{return failure('UNAVAILABLE','Service unavailable',503,id,undefined,head)}
}
export default {fetch(request:Request,env:Env){return handle(request,env)}} satisfies ExportedHandler<Env>;

export class SitemapService extends WorkerEntrypoint<Env>{fetch(request:Request){return sitemap(request,this.env);}}
// Only the Web Worker binds this entrypoint. No HTTP header can select it.
export class AuthService extends WorkerEntrypoint<Env>{fetch(request:Request){return handle(request,this.env,true);}}
