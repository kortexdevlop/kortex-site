import {z} from 'zod';
export interface Env {
 DB:D1Database;
 TOOLS_BUCKET:R2Bucket;
 KORTEX_ENV:'local'|'staging'|'production';
 PUBLIC_WEB_ORIGIN:string;
 CURSOR_SIGNING_KEY?:string;
 ADMIN_AUTH_POLICY?:string;
 OIDC_ENABLED?:string;
 OIDC_ISSUER?:string;
 OIDC_WEB_CLIENT_ID?:string;
 OIDC_DESKTOP_CLIENT_ID?:string;
 OIDC_WEB_REDIRECT_URI?:string;
 OIDC_DESKTOP_REDIRECT_URI?:string;
 OIDC_ENDPOINT_ORIGINS?:string;
 OIDC_WEB_CLIENT_SECRET?:string;
 SESSION_HMAC_SECRET?:string;
 PASSWORD_AUTH_ENABLED?:string;
 PASSWORD_PEPPER?:string;
 AI?:{run(model:string,input:Record<string,unknown>):Promise<unknown>};
 AI_ENABLED?:string;
 AI_MODEL?:string;
 AI_USER_DAILY_REQUESTS?:string;
 AI_GLOBAL_DAILY_REQUESTS?:string;
 AI_DAILY_RESERVED_TOKENS?:string;
}
const config=z.object({KORTEX_ENV:z.enum(['local','staging','production']),PUBLIC_WEB_ORIGIN:z.url()}).superRefine((value,ctx)=>{
 const url=new URL(value.PUBLIC_WEB_ORIGIN);
 const local=value.KORTEX_ENV==='local';
 if(url.origin!==value.PUBLIC_WEB_ORIGIN||url.username||url.password||url.hostname.endsWith('.invalid')||
  (!local&&url.protocol!=='https:')||
  (local&&!['http:','https:'].includes(url.protocol))||
  (local&&!['localhost','127.0.0.1','[::1]'].includes(url.hostname)))ctx.addIssue({code:'custom',message:'Invalid web origin'});
});
function methods(value:unknown,names:string[]):boolean {
 return value!==null&&typeof value==='object'&&names.every(name=>typeof (value as Record<string,unknown>)[name]==='function');
}
export function validateEnv(value:unknown):Env|null {
 if(value===null||typeof value!=='object')return null;
 const v=value as Record<string,unknown>;
 if(!config.safeParse(v).success||!methods(v.DB,['prepare','batch'])||!methods(v.TOOLS_BUCKET,['head','get','put']))return null;
 const identity:Partial<Env>={};for(const name of ['OIDC_ENABLED','OIDC_ISSUER','OIDC_WEB_CLIENT_ID','OIDC_DESKTOP_CLIENT_ID','OIDC_WEB_REDIRECT_URI','OIDC_DESKTOP_REDIRECT_URI','OIDC_ENDPOINT_ORIGINS','OIDC_WEB_CLIENT_SECRET','SESSION_HMAC_SECRET','PASSWORD_AUTH_ENABLED','PASSWORD_PEPPER'] as const)if(typeof v[name]==='string')identity[name]=v[name];
 for(const name of ['AI_ENABLED','AI_MODEL','AI_USER_DAILY_REQUESTS','AI_GLOBAL_DAILY_REQUESTS','AI_DAILY_RESERVED_TOKENS'] as const)if(typeof v[name]==='string')identity[name]=v[name];if(methods(v.AI,['run']))identity.AI=v.AI as Env['AI'];
 return {...identity,KORTEX_ENV:v.KORTEX_ENV as Env['KORTEX_ENV'],PUBLIC_WEB_ORIGIN:v.PUBLIC_WEB_ORIGIN as string,DB:v.DB as D1Database,TOOLS_BUCKET:v.TOOLS_BUCKET as R2Bucket,CURSOR_SIGNING_KEY:typeof v.CURSOR_SIGNING_KEY==='string'?v.CURSOR_SIGNING_KEY:undefined,ADMIN_AUTH_POLICY:typeof v.ADMIN_AUTH_POLICY==='string'?v.ADMIN_AUTH_POLICY:undefined};
}
