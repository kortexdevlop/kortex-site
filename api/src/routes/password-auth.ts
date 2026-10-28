import {z} from 'zod';
import type {Env} from '../env';
import {json,failure} from '../lib/http';
import {readBounded,AdminError} from '../lib/admin-error';
import {opaqueHash,randomToken,cookie,setCookie,rotateSession,SESSION_COOKIE,SESSION_TTL,secret,encoder} from '../security/sessions';
import {equal} from '../security/csrf';
import {passwordReady,passwordPepper,passwordEmailKey,validNewPassword,hashPassword,verifyPassword,PasswordBusy} from '../security/passwords';
const FORM_COOKIE='__Host-kortex_password',FORM_TTL=600000;
const paths=new Set(['/api/v1/auth/options','/api/v1/auth/password/token','/api/v1/auth/password/register','/api/v1/auth/password/login']);
const inputSchema=z.object({email:z.string().max(254),password:z.string().min(1).max(256)}).strict();
export function normalizeEmail(value:string):string|null{const email=value.trim().toLowerCase();return email.length<=254&&/^[\x21-\x7e]+$/.test(email)&&z.email().safeParse(email).success&&email.split('@')[0].length<=64?email:null}
async function rate(env:Env,key:string,limit:number,windowMs:number):Promise<boolean>{
 const now=Date.now(),window=Math.floor(now/windowMs),bucket=await opaqueHash(env,'kortex-password-rate-v1',key+':'+window);
 const result=await env.DB.prepare('INSERT INTO password_auth_limits VALUES(?,1,?) ON CONFLICT(bucket) DO UPDATE SET attempts=attempts+1 RETURNING attempts').bind(bucket,(window+1)*windowMs).first<{attempts:number}>();return !!result&&result.attempts<=limit;
}
export async function passwordAuth(request:Request,env:Env,id:string,viaWeb:boolean):Promise<Response|null>{
 const url=new URL(request.url);if(!paths.has(url.pathname))return null;if(!viaWeb)return failure('NOT_FOUND','Route not found',404,id);
 const options=url.pathname.endsWith('/options'),token=url.pathname.endsWith('/token'),register=url.pathname.endsWith('/register');
 if(request.method!==(options||token?'GET':'POST'))return failure('BAD_REQUEST','Method not allowed',405,id,undefined,false,{Allow:options||token?'GET':'POST'});
 if(url.searchParams.size)return failure('BAD_REQUEST','Invalid request',400,id);
 if(options)return json({password:passwordReady(env),google:env.OIDC_ENABLED==='true'&&!!env.OIDC_WEB_CLIENT_ID&&!!env.OIDC_WEB_CLIENT_SECRET&&!!env.SESSION_HMAC_SECRET},200,id);
 if(!passwordReady(env))return failure('UNAVAILABLE','Email sign-in is not configured',503,id);
 try{
  secret(env);passwordPepper(env);
  if(request.headers.get('Sec-Fetch-Site')==='cross-site'||request.headers.get('Origin')&&request.headers.get('Origin')!==env.PUBLIC_WEB_ORIGIN)return failure('FORBIDDEN','Invalid request origin',403,id);
  await env.DB.prepare('DELETE FROM password_auth_limits WHERE expires_at<=?').bind(Date.now()).run();
  if(!await rate(env,token?'forms-global':'password-global',token?120:60,60000))return failure('RATE_LIMITED','Please try again later',429,id,undefined,false,{'Retry-After':'60'});
  if(token){
   const value=randomToken(),hash=await opaqueHash(env,'kortex-password-form-v1',value);
   await env.DB.prepare('DELETE FROM password_auth_forms WHERE expires_at<=?').bind(Date.now()).run();
   const inserted=await env.DB.prepare('INSERT INTO password_auth_forms SELECT ?,? WHERE (SELECT count(*) FROM password_auth_forms)<10000').bind(hash,Date.now()+FORM_TTL).run();
   if(inserted.meta.changes!==1)return failure('RATE_LIMITED','Please try again later',429,id);
   return json({csrf_token:await opaqueHash(env,'kortex-password-csrf-v1',value)},200,id,undefined,false,{'Set-Cookie':setCookie(FORM_COOKIE,value,FORM_TTL/1000)});
  }
  const browser=cookie(request,FORM_COOKIE),csrf=request.headers.get('X-CSRF-Token');
  if(request.headers.get('Origin')!==env.PUBLIC_WEB_ORIGIN||!browser||!csrf||csrf.length!==64||!equal(csrf,await opaqueHash(env,'kortex-password-csrf-v1',browser)))return failure('FORBIDDEN','Invalid request verification',403,id);
  const formHash=await opaqueHash(env,'kortex-password-form-v1',browser),form=await env.DB.prepare('SELECT token_hash FROM password_auth_forms WHERE token_hash=? AND expires_at>?').bind(formHash,Date.now()).first();if(!form)return failure('FORBIDDEN','Invalid request verification',403,id);
  if(!request.headers.get('Content-Type')?.startsWith('application/json'))return failure('BAD_REQUEST','JSON request required',415,id);
  const parsed=inputSchema.safeParse(JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(await readBounded(request.body,4096))));
  if(!parsed.success)return failure('BAD_REQUEST','Invalid email or password',400,id);
  const {password}=parsed.data,email=normalizeEmail(parsed.data.email);if(!email||[...password].length>128||encoder.encode(password).length>512)return failure('BAD_REQUEST','Invalid email or password',400,id);
  const emailKey=await passwordEmailKey(env,email);
  if(!await rate(env,(register?'register:':'login:')+emailKey,register?3:8,register?3600000:900000))return failure('RATE_LIMITED','Please try again later',429,id,undefined,false,{'Retry-After':register?'3600':'900'});
  let userId:string;
  if(register){
   if(!validNewPassword(password))return failure('BAD_REQUEST','Use 15 to 128 characters and avoid common passwords',400,id);
   const hash=await hashPassword(env,password),candidate=crypto.randomUUID(),now=Date.now();
   const results=await env.DB.batch([env.DB.prepare('INSERT INTO users(id,created_at) SELECT ?,? WHERE NOT EXISTS(SELECT 1 FROM password_credentials WHERE email_key=?)').bind(candidate,now,emailKey),env.DB.prepare('INSERT INTO password_credentials(email_key,email,user_id,password_hash,created_at) SELECT ?,?,?,?,? WHERE EXISTS(SELECT 1 FROM users WHERE id=?)').bind(emailKey,email,candidate,hash,now,candidate)]);
   if(results[0].meta.changes!==1)return failure('CONFLICT','Unable to create an account with these details',409,id);
   userId=candidate;
  }else{
   const credentials=await env.DB.prepare('SELECT user_id,password_hash FROM password_credentials WHERE email_key=?').bind(emailKey).first<{user_id:string;password_hash:string}>();
   if(!await verifyPassword(env,password,credentials?.password_hash??null)||!credentials)return failure('UNAUTHORIZED','Invalid email or password',401,id);
   userId=credentials.user_id;
  }
  const session=await rotateSession(request,env,userId);
  await env.DB.prepare('DELETE FROM password_auth_forms WHERE token_hash=?').bind(formHash).run();
  const headers=new Headers();headers.append('Set-Cookie',setCookie(SESSION_COOKIE,session,SESSION_TTL/1000));headers.append('Set-Cookie',setCookie(FORM_COOKIE,'',0));
  return json({ok:true},register?201:200,id,undefined,false,headers);
 }catch(error){
  if(error instanceof AdminError)return failure(error.code,'Invalid request',error.status,id);
  if(error instanceof PasswordBusy)return failure('RATE_LIMITED','Please try again shortly',429,id,undefined,false,{'Retry-After':'5'});
  if(error instanceof SyntaxError||error instanceof TypeError)return failure('BAD_REQUEST','Invalid request',400,id);
  return failure('UNAVAILABLE','Email sign-in temporarily unavailable',503,id);
 }
}
