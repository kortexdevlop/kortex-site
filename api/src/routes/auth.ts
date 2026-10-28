import type {Env} from '../env';
import {json,failure} from '../lib/http';
import {configuration,exchangeCode,verifyIdToken,resolveUser,desktopIdentity} from '../services/identity';
import {discoverIdentity} from '../services/identity-config';
import {expireIdentityData} from '../services/identity-store';
import {b64,randomToken,cookie,setCookie,SESSION_COOKIE,FLOW_COOKIE,FLOW_TTL,SESSION_TTL,opaqueHash,encryptVerifier,decryptVerifier,rotateSession,session,encoder,secret} from '../security/sessions';
import {checkCsrf,csrfToken,equal} from '../security/csrf';
type Flow={nonce_hash:string;verifier_encrypted:string;config_hash:string};
const paths=new Set(['/api/v1/auth/login','/api/v1/auth/callback','/api/v1/auth/logout','/api/v1/me']);
export async function auth(request:Request,env:Env,id:string,viaWeb=false,transport:typeof fetch=fetch):Promise<Response|null>{
 const url=new URL(request.url);if(!paths.has(url.pathname))return null;
 const logout=url.pathname.endsWith('/logout'),callback=url.pathname.endsWith('/callback'),me=url.pathname==='/api/v1/me';
 if(request.method!==(logout?'POST':'GET'))return failure('BAD_REQUEST','Method not allowed',405,id,undefined,false,{Allow:logout?'POST':'GET'});
 if(!viaWeb&&(!me||!request.headers.has('Authorization')))return failure('NOT_FOUND','Route not found',404,id);
 const clear=()=>setCookie(FLOW_COOKIE,'',0);
 try{
  if(env.OIDC_ENABLED!=='true'&&env.PASSWORD_AUTH_ENABLED!=='true')return failure('UNAVAILABLE','Sign-in is not configured',503,id);
  secret(env); // Fail before storing flow data when server secrets are unavailable.
  if(url.href.length>8192)throw Error('IDENTITY_QUERY');
  const names=[...url.searchParams.keys()];if(new Set(names).size!==names.length||names.some(n=>!(callback?['code','state','error','error_description','iss']:[]).includes(n)))throw Error('IDENTITY_QUERY');
  if(me){
   let userId:string,csrf:string|undefined,authentication:string,scopes:readonly string[];
   if(request.headers.has('Authorization')){const config=configuration(env);if(!config)throw Error('IDENTITY_CONFIG');const identity=await desktopIdentity(request,env,config,transport);userId=identity.userId;scopes=identity.scopes;authentication='desktop';}
   else{const current=await session(request,env);if(!current)throw Error('IDENTITY_SESSION');userId=current.user_id;csrf=await csrfToken(env,current.token);scopes=['identity:read','session:logout'];authentication='web';}
   const user=await env.DB.prepare('SELECT u.id,u.sync_enabled,p.email,p.email_verified FROM users u LEFT JOIN password_credentials p ON p.user_id=u.id WHERE u.id=?').bind(userId).first<{id:string;sync_enabled:number;email:string|null;email_verified:number|null}>();if(!user)throw Error('IDENTITY_USER');
   return json({user:{id:user.id,sync_enabled:!!user.sync_enabled,...(user.email?{email:user.email,email_verified:!!user.email_verified}:{})},authentication,scopes,...(csrf?{csrf_token:csrf}:{})},200,id);
  }
  if(logout){
   if(request.headers.has('Authorization'))return failure('FORBIDDEN','Cookie session required',403,id);
   const current=await session(request,env);if(!current)throw Error('IDENTITY_SESSION');
   if(!await checkCsrf(request,env,current.token))return failure('FORBIDDEN','Invalid request verification',403,id);
   await env.DB.prepare('UPDATE sessions SET revoked_at=? WHERE token_hash=? AND revoked_at IS NULL').bind(Date.now(),current.hash).run();
   return json(null,204,id,undefined,false,{'Set-Cookie':setCookie(SESSION_COOKIE,'',0)});
  }
  if(!callback){
   const config=configuration(env);if(!config)return failure('UNAVAILABLE','Google sign-in is not configured',503,id);
   if(request.headers.get('Sec-Fetch-Site')==='cross-site'||request.headers.get('Origin')&&request.headers.get('Origin')!==env.PUBLIC_WEB_ORIGIN) return failure('FORBIDDEN','Invalid sign-in origin',403,id);
   const discovery=await discoverIdentity(config,env.KORTEX_ENV,transport);
   if(!discovery.metadata.id_token_signing_alg_values_supported.includes('RS256'))throw Error('IDENTITY_ALGORITHM');
   const state=randomToken(),nonce=randomToken(),browser=randomToken(),verifier=randomToken(),now=Date.now();
   const stateHash=await opaqueHash(env,'kortex-state-v1',state),browserHash=await opaqueHash(env,'kortex-browser-v1',browser),nonceHash=await opaqueHash(env,'kortex-nonce-v1',nonce),configHash=await opaqueHash(env,'kortex-config-v1',JSON.stringify(config));
   await expireIdentityData(env.DB,now);
   await env.DB.prepare('DELETE FROM oidc_flows WHERE expires_at<=?').bind(now).run();
   const inserted=await env.DB.prepare('INSERT INTO oidc_flows SELECT ?,?,?,?,?,?,? WHERE (SELECT count(*) FROM oidc_flows)<10000').bind(stateHash,browserHash,nonceHash,await encryptVerifier(env,verifier),configHash,now,now+FLOW_TTL).run();
   if(inserted.meta.changes!==1)return failure('RATE_LIMITED','Sign-in temporarily unavailable',429,id);
   const target=new URL(discovery.metadata.authorization_endpoint);target.search=new URLSearchParams({client_id:config.webClientId,redirect_uri:config.webRedirect,response_type:'code',scope:'openid',state,nonce,code_challenge:b64(new Uint8Array(await crypto.subtle.digest('SHA-256',encoder.encode(verifier)))),code_challenge_method:'S256'}).toString();
   return json(null,302,id,undefined,false,{Location:target.href,'Set-Cookie':setCookie(FLOW_COOKIE,browser,FLOW_TTL/1000)});
  }
  const config=configuration(env);if(!config)return failure('UNAVAILABLE','Google sign-in is not configured',503,id);
  const state=url.searchParams.get('state'),browser=cookie(request,FLOW_COOKIE);
  if(!state||!/^[A-Za-z0-9_-]{43}$/.test(state)||!browser)throw Error('IDENTITY_STATE');
  // One atomic DELETE..RETURNING, before token exchange: concurrent callbacks cannot replay a flow.
  const flow=await env.DB.prepare('DELETE FROM oidc_flows WHERE state_hash=? AND browser_hash=? AND expires_at>? RETURNING nonce_hash,verifier_encrypted,config_hash').bind(await opaqueHash(env,'kortex-state-v1',state),await opaqueHash(env,'kortex-browser-v1',browser),Date.now()).first<Flow>();
  if(!flow||!equal(flow.config_hash,await opaqueHash(env,'kortex-config-v1',JSON.stringify(config)))||url.searchParams.has('error')||url.searchParams.has('iss')&&url.searchParams.get('iss')!==config.issuer)throw Error('IDENTITY_STATE');
  const code=url.searchParams.get('code');if(!code||code.length>4096)throw Error('IDENTITY_CODE');
  const discovery=await discoverIdentity(config,env.KORTEX_ENV,transport),token=await exchangeCode(env,config,discovery,code,await decryptVerifier(env,flow.verifier_encrypted),transport),claims=await verifyIdToken(token,config,discovery,config.webClientId);
  if(!claims.nonce||!equal(flow.nonce_hash,await opaqueHash(env,'kortex-nonce-v1',claims.nonce)))throw Error('IDENTITY_NONCE');
  const user=await resolveUser(env,claims.iss,claims.sub),newSession=await rotateSession(request,env,user);
  const headers=new Headers({Location:env.PUBLIC_WEB_ORIGIN+'/app'});headers.append('Set-Cookie',setCookie(SESSION_COOKIE,newSession,SESSION_TTL/1000));headers.append('Set-Cookie',clear());
  return json(null,303,id,undefined,false,headers);
 }catch{
  return failure('UNAUTHORIZED','Sign-in or session verification failed',401,id,undefined,false,callback?{'Set-Cookie':clear()}:undefined);
 }
}
