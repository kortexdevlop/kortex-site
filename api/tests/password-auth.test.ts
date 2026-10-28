import {env} from 'cloudflare:workers';
import {applyD1Migrations} from 'cloudflare:test';
import {beforeAll,beforeEach,it,expect} from 'vitest';
import type {Env} from '../src/env';
import {handle} from '../src/index';
import {normalizeEmail} from '../src/routes/password-auth';
import {hashPassword,verifyPassword,passwordEmailKey,validNewPassword} from '../src/security/passwords';
import {opaqueHash} from '../src/security/sessions';
import {resolveUser} from '../src/services/identity';
import {proxy} from '../../website/src/lib/proxy';
import vector from '../../tests/fixtures/password-scrypt-test.json';
const web='https://password.kortex.test',email='password-tests@example.test',password=vector.password;
const settings=():Env=>({...env,KORTEX_ENV:'staging',PUBLIC_WEB_ORIGIN:web,OIDC_ENABLED:'false',PASSWORD_AUTH_ENABLED:'true',PASSWORD_PEPPER:vector.pepper,SESSION_HMAC_SECRET:'test-only-password-session-key-value'});
const jar=(r:Response)=>r.headers.getSetCookie().filter(c=>!c.includes('Max-Age=0')).map(c=>c.split(';')[0]).join('; ');
beforeAll(async()=>{await applyD1Migrations(env.DB,env.TEST_MIGRATIONS)});
beforeEach(async()=>{await env.DB.prepare('DELETE FROM users WHERE id IN(SELECT user_id FROM password_credentials)').run();await env.DB.prepare('DELETE FROM password_auth_forms').run();await env.DB.prepare('DELETE FROM password_auth_limits').run()});
const call=(path:string,init:RequestInit={})=>proxy(new Request(web+path,init),undefined,web,{fetch:r=>handle(r,settings(),true)});
async function form(){const response=await call('/api/v1/auth/password/token');expect(response.status).toBe(200);const body=await response.json() as {csrf_token:string};return {cookie:jar(response),csrf:body.csrf_token}}
async function post(kind='register',credentials={email,password},proof?:Awaited<ReturnType<typeof form>>){const f=proof??await form();return call('/api/v1/auth/password/'+kind,{method:'POST',headers:{Cookie:f.cookie,Origin:web,'Content-Type':'application/json','X-CSRF-Token':f.csrf},body:JSON.stringify(credentials)})}
it('normalizes emails conservatively and accepts long Unicode phrases without trimming passwords',()=>{
 expect(normalizeEmail('  User+tag@EXAMPLE.COM ')).toBe('user+tag@example.com');expect(normalizeEmail('a.b@example.com')).not.toBe(normalizeEmail('ab@example.com'));for(const bad of ['x','a b@example.com','a@example.com\r\nBcc:x','é@example.com','a'.repeat(65)+'@example.com'])expect(normalizeEmail(bad)).toBeNull();expect(validNewPassword(password)).toBe(true);for(const bad of ['short','passwordpassword','x'.repeat(15),' '.repeat(20),'a'.repeat(129)])expect(validNewPassword(bad)).toBe(false);
});
it('matches an independent native scrypt vector and never treats wrong or malformed hashes as valid',async()=>{
 expect(await verifyPassword(settings(),password,vector.stored)).toBe(true);expect(await verifyPassword(settings(),password+'!',vector.stored)).toBe(false);expect(await verifyPassword(settings(),password,vector.stored.replace('$32768$','$1$'))).toBe(false);const first=await hashPassword(settings(),password),second=await hashPassword(settings(),password);expect(first).not.toBe(second);expect(first).not.toContain(password);expect(await verifyPassword(settings(),password,first)).toBe(true);
});
it('registers and starts an independent cookie session with Google disabled; no verified-email claim',async()=>{
 const response=await post();expect(response.status).toBe(201);expect(response.headers.get('Set-Cookie')).toContain('Secure; HttpOnly; SameSite=Lax');const cookie=jar(response);const me=await call('/api/v1/me',{headers:{Cookie:cookie}});expect(me.status).toBe(200);const body=await me.json() as {user:{id:string;email:string;email_verified:boolean;sync_enabled:boolean};csrf_token:string};expect(body.user.email).toBe(email);expect(body.user.email_verified).toBe(false);expect(body.user.sync_enabled).toBe(false);
 const row=await env.DB.prepare('SELECT * FROM password_credentials WHERE user_id=?').bind(body.user.id).first();expect(row?.password_hash).toMatch(/^scrypt\$v1\$32768\$8\$3\$/);expect(JSON.stringify(row)).not.toContain(password);expect(JSON.stringify(body)).not.toContain('scrypt');expect((await env.DB.prepare('SELECT count(*) AS n FROM external_identities WHERE user_id=?').bind(body.user.id).first())?.n).toBe(0);
 const logout=await call('/api/v1/auth/logout',{method:'POST',headers:{Cookie:cookie,Origin:web,'X-CSRF-Token':body.csrf_token}});expect(logout.status).toBe(204);expect((await call('/api/v1/me',{headers:{Cookie:cookie}})).status).toBe(401);
});
it('logs in with normalized email, rejects unknown and wrong credentials equally, and prevents duplicates',async()=>{
 expect((await post()).status).toBe(201);expect((await post('register',{email:' PASSWORD-TESTS@EXAMPLE.TEST ',password})).status).toBe(409);
 const wrong=await post('login',{email,password:'Un autre mot de passe faux'}),unknown=await post('login',{email:'unknown@example.test',password});expect(wrong.status).toBe(401);expect(unknown.status).toBe(401);const a=await wrong.json() as {error:{message:string}},b=await unknown.json() as {error:{message:string}};expect(a.error.message).toBe(b.error.message);
 const valid=await post('login',{email:email.toUpperCase(),password});expect(valid.status).toBe(200);expect((await call('/api/v1/me',{headers:{Cookie:jar(valid)}})).status).toBe(200);expect((await env.DB.prepare('SELECT count(*) AS n FROM password_credentials').first())?.n).toBe(1);
});
it('does not link a password account to an existing Google identity',async()=>{
 const google=await resolveUser(settings(),'https://accounts.google.com','password-link-test');const response=await post(),body=await(await call('/api/v1/me',{headers:{Cookie:jar(response)}})).json() as {user:{id:string}};expect(body.user.id).not.toBe(google);expect((await env.DB.prepare('SELECT count(*) AS n FROM external_identities WHERE user_id=?').bind(body.user.id).first())?.n).toBe(0);await env.DB.prepare('DELETE FROM users WHERE id=?').bind(google).run();
});
it('requires a live browser-bound form token, Origin and CSRF proof and blocks direct API cookie access',async()=>{
 const proof=await form();for(const headers of [{Origin:web},{Origin:'https://evil.test',Cookie:proof.cookie,'X-CSRF-Token':proof.csrf},{Origin:web,Cookie:proof.cookie,'X-CSRF-Token':'0'.repeat(64)},{Origin:web,Cookie:proof.cookie+'; '+proof.cookie,'X-CSRF-Token':proof.csrf}] as Record<string,string>[])expect((await call('/api/v1/auth/password/register',{method:'POST',headers:{...headers,'Content-Type':'application/json'},body:JSON.stringify({email,password})})).status).toBe(403);
 await env.DB.prepare('UPDATE password_auth_forms SET expires_at=?').bind(Date.now()-1).run();expect((await post('register',{email,password},proof)).status).toBe(403);expect((await handle(new Request(web+'/api/v1/auth/password/token'),settings())).status).toBe(404);
});
it('enforces persistent pre-hash rate limits for an email and for the global request budget',async()=>{
 const proof=await form(),now=Date.now(),window=900000,key=await passwordEmailKey(settings(),email),bucket=await opaqueHash(settings(),'kortex-password-rate-v1','login:'+key+':'+Math.floor(now/window));await env.DB.prepare('INSERT INTO password_auth_limits VALUES(?,8,?)').bind(bucket,now+window).run();const denied=await post('login',{email,password},proof);expect(denied.status).toBe(429);expect(denied.headers.get('Retry-After')).toBe('900');
 const global=await opaqueHash(settings(),'kortex-password-rate-v1','password-global:'+Math.floor(now/60000));await env.DB.prepare('UPDATE password_auth_limits SET attempts=60 WHERE bucket=?').bind(global).run();expect((await post('register',{email:'another@example.test',password},proof)).status).toBe(429);
});
it('bounds payloads and rejects weak passwords, unknown fields, query credentials and malformed JSON',async()=>{
 expect((await post('register',{email,password:'short'})).status).toBe(400);expect((await call('/api/v1/auth/password/login?password=never-in-url',{method:'POST'})).status).toBe(403);const proof=await form();
 for(const body of ['{',JSON.stringify({email,password,email_verified:true}),JSON.stringify({email,password:'x'.repeat(5000)})]){const response=await call('/api/v1/auth/password/register',{method:'POST',headers:{Origin:web,Cookie:proof.cookie,'X-CSRF-Token':proof.csrf,'Content-Type':'application/json'},body});expect([400,403,413]).toContain(response.status);expect(await response.text()).not.toContain(password)}
});
it('stays disabled without both server secrets, leaves Google separately configurable and cascades account deletion',async()=>{
 const disabled=await handle(new Request(web+'/api/v1/auth/options'),{...settings(),PASSWORD_PEPPER:undefined},true),options=await disabled.json() as {password:boolean;google:boolean};expect(options).toEqual({password:false,google:false});const response=await post(),cookie=jar(response),body=await(await call('/api/v1/me',{headers:{Cookie:cookie}})).json() as {user:{id:string}};await env.DB.prepare('DELETE FROM users WHERE id=?').bind(body.user.id).run();expect((await call('/api/v1/me',{headers:{Cookie:cookie}})).status).toBe(401);expect((await env.DB.prepare('SELECT count(*) AS n FROM password_credentials').first())?.n).toBe(0);
});
