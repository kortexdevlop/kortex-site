import {env} from 'cloudflare:workers';
import {applyD1Migrations} from 'cloudflare:test';
import {beforeAll,beforeEach,it,expect} from 'vitest';
import type {Env} from '../src/env';
import {handle} from '../src/index';
import {auth} from '../src/routes/auth';
import {configuration,verifyIdToken} from '../src/services/identity';
import {discoverIdentity} from '../src/services/identity-config';
import {sessionHash} from '../src/services/identity-store';
import {b64,randomToken,SESSION_COOKIE,FLOW_COOKIE} from '../src/security/sessions';
import {proxy} from '../../website/src/lib/proxy';
const web='https://kortex.test',issuer='https://accounts.google.com';
let signing:CryptoKeyPair,jwk:JsonWebKey;
const settings=():Env=>({...env,KORTEX_ENV:'staging',PUBLIC_WEB_ORIGIN:web,OIDC_ENABLED:'true',OIDC_ISSUER:issuer,OIDC_WEB_CLIENT_ID:'web-test',OIDC_DESKTOP_CLIENT_ID:'desktop-test',OIDC_WEB_REDIRECT_URI:web+'/auth/callback',OIDC_DESKTOP_REDIRECT_URI:'http://127.0.0.1:49152/callback',OIDC_ENDPOINT_ORIGINS:issuer,OIDC_WEB_CLIENT_SECRET:'test-only-not-a-real-google-secret',SESSION_HMAC_SECRET:'test-only-ephemeral-session-hmac-value'});
const metadata={issuer,authorization_endpoint:issuer+'/authorize',token_endpoint:issuer+'/token',jwks_uri:issuer+'/jwks',response_types_supported:['code'],subject_types_supported:['public'],id_token_signing_alg_values_supported:['RS256'],code_challenge_methods_supported:['S256']};
const encoder=new TextEncoder();const codes=new Map<string,{nonce:string;challenge:string;claims?:Record<string,unknown>}>();let exchanges=0;
async function jwt(claims:Record<string,unknown>={},header:Record<string,unknown>={}){
 const now=Math.floor(Date.now()/1000),head=b64(encoder.encode(JSON.stringify({alg:'RS256',kid:'local-idp',typ:'JWT',...header}))),body=b64(encoder.encode(JSON.stringify({iss:issuer,sub:'auth-test-subject',aud:'web-test',iat:now,exp:now+3600,...claims}))),message=head+'.'+body;
 return message+'.'+b64(new Uint8Array(await crypto.subtle.sign('RSASSA-PKCS1-v1_5',signing.privateKey,encoder.encode(message))));
}
// Controlled identity provider transport; keys and signatures are real, no validation is bypassed.
const provider=(async(input:RequestInfo|URL,init?:RequestInit)=>{
 expect(init?.redirect).toBe('error');const url=String(input);
 if(url.endsWith('/.well-known/openid-configuration'))return Response.json(metadata);
 if(url.endsWith('/jwks'))return Response.json({keys:[{...jwk,kid:'local-idp',alg:'RS256',use:'sig'}]});
 if(url.endsWith('/token')){
  exchanges++;const body=new URLSearchParams(String(init?.body)),code=body.get('code')??'',entry=codes.get(code);codes.delete(code);
  expect(body.get('client_id')).toBe('web-test');expect(body.get('redirect_uri')).toBe(web+'/auth/callback');expect(body.get('grant_type')).toBe('authorization_code');expect(body.get('client_secret')).toBe(settings().OIDC_WEB_CLIENT_SECRET);
  const challenge=b64(new Uint8Array(await crypto.subtle.digest('SHA-256',encoder.encode(body.get('code_verifier')??''))));
  if(!entry||challenge!==entry.challenge)return Response.json({error:'invalid_grant'},{status:400});
  return Response.json({id_token:await jwt({nonce:entry.nonce,...entry.claims}),access_token:'not-exposed-by-kortex',token_type:'Bearer'});
 }
 throw Error('Unexpected provider URL');
}) as typeof fetch;
beforeAll(async()=>{await applyD1Migrations(env.DB,env.TEST_MIGRATIONS);signing=await crypto.subtle.generateKey({name:'RSASSA-PKCS1-v1_5',modulusLength:2048,publicExponent:new Uint8Array([1,0,1]),hash:'SHA-256'},true,['sign','verify']) as CryptoKeyPair;jwk=await crypto.subtle.exportKey('jwk',signing.publicKey) as JsonWebKey});
beforeEach(async()=>{codes.clear();exchanges=0;await env.DB.prepare('DELETE FROM oidc_flows').run();await env.DB.prepare('DELETE FROM users WHERE id IN (SELECT user_id FROM external_identities WHERE issuer=?)').bind(issuer).run()});
function call(path:string,init:RequestInit={}){const binding={async fetch(r:Request){return (await auth(r,settings(),crypto.randomUUID(),true,provider))!}};return proxy(new Request(web+path,init),undefined,web,binding)}
const cookies=(response:Response)=>response.headers.getSetCookie().filter(c=>!c.includes('Max-Age=0')).map(c=>c.split(';')[0]).join('; ');
async function begin(claims?:Record<string,unknown>){const response=await call('/api/v1/auth/login');expect(response.status).toBe(302);const target=new URL(response.headers.get('Location')!),code=randomToken();codes.set(code,{nonce:target.searchParams.get('nonce')!,challenge:target.searchParams.get('code_challenge')!,claims});return {response,target,code,cookie:cookies(response),state:target.searchParams.get('state')!}}
const callback=(flow:Awaited<ReturnType<typeof begin>>,cookie=flow.cookie,extra='')=>call('/api/v1/auth/callback?code='+flow.code+'&state='+flow.state+extra,{headers:{Cookie:cookie,'Sec-Fetch-Site':'cross-site'}});
async function login(){const flow=await begin(),response=await callback(flow);expect(response.status).toBe(303);return {flow,response,cookie:cookies(response)}}
it('stays disabled without client configuration and cannot select the Web entrypoint with headers',async()=>{
 const disabled=await auth(new Request(web+'/api/v1/auth/login'),{...settings(),OIDC_ENABLED:'false'},crypto.randomUUID(),true,provider);expect(disabled?.status).toBe(503);
 const direct=await handle(new Request(web+'/api/v1/auth/login',{headers:{'X-Kortex-Auth-Service':'true','X-Forwarded-Host':web}}),settings());expect(direct.status).toBe(404);expect(direct.headers.has('Set-Cookie')).toBe(false);
});
it('creates browser-bound hashed state, encrypted verifier, S256 and a secure host-only flow cookie',async()=>{
 const flow=await begin();expect(flow.target.origin).toBe(issuer);expect(flow.target.searchParams.get('scope')).toBe('openid');expect(flow.target.searchParams.get('code_challenge_method')).toBe('S256');
 expect(flow.response.headers.get('Set-Cookie')).toContain('Secure; HttpOnly; SameSite=Lax');expect(flow.response.headers.get('Set-Cookie')).not.toContain('Domain=');
 const row=await env.DB.prepare('SELECT * FROM oidc_flows').first();expect(JSON.stringify(row)).not.toContain(flow.state);expect(JSON.stringify(row)).not.toContain(flow.target.searchParams.get('nonce'));expect(row?.verifier_encrypted).toMatch(/^[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/);
});
it('exchanges code with PKCE, validates signatures and creates an opaque session without exposing provider tokens',async()=>{
 const {response,cookie}=await login();expect(response.headers.get('Location')).toBe(web+'/app');expect(response.headers.get('Referrer-Policy')).toBe('no-referrer');expect(response.headers.getSetCookie()).toHaveLength(2);
 const sessionCookie=response.headers.getSetCookie().find(c=>c.startsWith(SESSION_COOKIE+'='))!;expect(sessionCookie).toContain('Path=/; Max-Age=604800; Secure; HttpOnly; SameSite=Lax');expect(sessionCookie).not.toContain('Domain=');expect(await response.text()).not.toContain('not-exposed-by-kortex');
 const me=await call('/api/v1/me',{headers:{Cookie:cookie}}),body=await me.json() as {user:{id:string;sync_enabled:boolean};csrf_token:string};expect(me.status).toBe(200);expect(body.user.sync_enabled).toBe(false);expect(body.csrf_token).toMatch(/^[a-f0-9]{64}$/);
 const rows=await env.DB.prepare('SELECT * FROM sessions').all();expect(rows.results).toHaveLength(1);expect(JSON.stringify(rows)).not.toContain(cookie.split('=')[1].split(';')[0]);expect(JSON.stringify(body)).not.toContain('access_token');expect(JSON.stringify(body)).not.toContain('web-test');
});
it('refuses browser/state mismatch, duplicate cookie and state replay, including concurrent callbacks',async()=>{
 const flow=await begin();expect((await callback(flow,FLOW_COOKIE+'='+randomToken())).status).toBe(401);expect((await callback(flow,flow.cookie+'; '+flow.cookie)).status).toBe(401);
 expect((await call('/api/v1/auth/callback?code='+flow.code+'&state='+randomToken(),{headers:{Cookie:flow.cookie}})).status).toBe(401);
 const responses=await Promise.all([callback(flow),callback(flow)]);expect(responses.map(r=>r.status).sort()).toEqual([303,401]);expect(exchanges).toBe(1);expect((await callback(flow)).status).toBe(401);expect(exchanges).toBe(1);
});
it('rejects bad nonce, issuer, audience, expiry, future issue time and ambiguous authorized party',async()=>{
 const now=Math.floor(Date.now()/1000);for(const claims of [{nonce:'wrong'},{iss:'https://evil.test'},{aud:'other'},{exp:now-1},{iat:now+100},{aud:['web-test','other']},{azp:'other'}]){const flow=await begin(claims);expect((await callback(flow)).status).toBe(401);expect((await callback(flow)).status).toBe(401)}
 expect((await env.DB.prepare('SELECT count(*) AS n FROM external_identities WHERE issuer=?').bind(issuer).first())?.n).toBe(0);
});
it('rejects forged signatures, none algorithm, attacker key URLs, unknown keys and web tokens used as desktop bearer',async()=>{
 const config=configuration(settings())!,discovery=await discoverIdentity(config,'staging',provider);const valid=await jwt();expect((await verifyIdToken(valid,config,discovery,'web-test')).sub).toBe('auth-test-subject');
 const parts=valid.split('.');parts[2]=(parts[2][0]==='a'?'b':'a')+parts[2].slice(1);await expect(verifyIdToken(parts.join('.'),config,discovery,'web-test')).rejects.toThrow();
 for(const header of [{alg:'none'},{kid:'unknown'},{jku:'https://evil.test/keys'},{crit:['unexpected']}])await expect(verifyIdToken(await jwt({},header),config,discovery,'web-test')).rejects.toThrow();
 await expect(verifyIdToken(valid,config,discovery,'desktop-test')).rejects.toThrow();
});
it('consumes expired or failed provider flows without issuing sessions or retrying a code',async()=>{
 const flow=await begin();await env.DB.prepare('UPDATE oidc_flows SET created_at=?,expires_at=?').bind(Date.now()-600000,Date.now()-1).run();expect((await callback(flow)).status).toBe(401);expect(exchanges).toBe(0);
 const denied=await begin();expect((await callback(denied,denied.cookie,'&error=access_denied')).status).toBe(401);expect((await callback(denied)).status).toBe(401);
 const invalid=await begin();codes.get(invalid.code)!.challenge='wrong';expect((await callback(invalid)).status).toBe(401);expect((await callback(invalid)).status).toBe(401);
});
it('requires same-origin CSRF verification for logout and revokes the session before clearing its cookie',async()=>{
 const user=await login(),me=await call('/api/v1/me',{headers:{Cookie:user.cookie}}),body=await me.json() as {csrf_token:string};
 for(const headers of [{Cookie:user.cookie,Origin:web},{Cookie:user.cookie,Origin:web,'X-CSRF-Token':'0'.repeat(64)},{Cookie:user.cookie,Origin:'https://evil.test','X-CSRF-Token':body.csrf_token}] as Record<string,string>[])expect((await call('/api/v1/auth/logout',{method:'POST',headers})).status).toBe(403);
 const response=await call('/api/v1/auth/logout',{method:'POST',headers:{Cookie:user.cookie,Origin:web,'X-CSRF-Token':body.csrf_token}});expect(response.status).toBe(204);expect(response.headers.get('Set-Cookie')).toContain('Max-Age=0');expect((await call('/api/v1/me',{headers:{Cookie:user.cookie}})).status).toBe(401);
});
it('rotates an existing session and retains exactly one identity when logging in twice',async()=>{
 const old=await login(),flow=await begin(),response=await callback(flow,flow.cookie+'; '+old.cookie);expect(response.status).toBe(303);expect((await call('/api/v1/me',{headers:{Cookie:old.cookie}})).status).toBe(401);expect((await call('/api/v1/me',{headers:{Cookie:cookies(response)}})).status).toBe(200);expect((await env.DB.prepare('SELECT count(*) AS n FROM external_identities WHERE issuer=?').bind(issuer).first())?.n).toBe(1);
});
it('restricts desktop identity bearer to a known user and read-only identity scope; no cookie grants',async()=>{
 await login();const token=await jwt({aud:'desktop-test'}),response=await auth(new Request('https://api.test/api/v1/me',{headers:{Authorization:'Bearer '+token}}),settings(),crypto.randomUUID(),false,provider);expect(response?.status).toBe(200);const body=await response!.json() as {scopes:string[];csrf_token?:string};expect(body.scopes).toEqual(['identity:read']);expect(body.csrf_token).toBeUndefined();expect(response?.headers.has('Set-Cookie')).toBe(false);
 expect((await call('/api/v1/me',{headers:{Authorization:'Bearer '+token}})).status).toBe(403);const invalid=await auth(new Request('https://api.test/api/v1/me',{headers:{Authorization:'Bearer '+await jwt()}}),settings(),crypto.randomUUID(),false,provider);expect(invalid?.status).toBe(401);
});
it('rejects unsupported methods and duplicate OAuth parameters without leaking configuration',async()=>{
 expect((await call('/api/v1/auth/login',{method:'POST'})).status).toBe(405);const flow=await begin();expect((await callback(flow,flow.cookie,'&state='+flow.state)).status).toBe(401);expect((await call('/api/v1/auth/login?returnTo=https://evil.test')).status).toBe(401);const body=await(await call('/api/v1/me')).text();expect(body).not.toContain(settings().OIDC_WEB_CLIENT_SECRET);expect(body).not.toContain('SELECT');
});
