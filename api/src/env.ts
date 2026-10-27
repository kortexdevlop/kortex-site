import {z} from 'zod';
export interface Env {
 DB:D1Database;
 TOOLS_BUCKET:R2Bucket;
 KORTEX_ENV:'local'|'staging'|'production';
 PUBLIC_WEB_ORIGIN:string;
 CURSOR_SIGNING_KEY?:string;
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
 return {KORTEX_ENV:v.KORTEX_ENV as Env['KORTEX_ENV'],PUBLIC_WEB_ORIGIN:v.PUBLIC_WEB_ORIGIN as string,DB:v.DB as D1Database,TOOLS_BUCKET:v.TOOLS_BUCKET as R2Bucket,CURSOR_SIGNING_KEY:typeof v.CURSOR_SIGNING_KEY==='string'?v.CURSOR_SIGNING_KEY:undefined};
}
