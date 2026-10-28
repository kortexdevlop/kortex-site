import {env} from 'cloudflare:workers';
import {applyD1Migrations} from 'cloudflare:test';
import {beforeAll,beforeEach,it,expect,vi} from 'vitest';
import type {Env} from '../src/env';
import {handle} from '../src/index';
import {sessionHash} from '../src/services/identity-store';
import {randomToken} from '../src/security/sessions';
import {csrfToken} from '../src/security/csrf';
import {proxy} from '../../website/src/lib/proxy';
const web='https://security.kortex.test',a='18020000-0000-4000-8000-000000000001',b='18020000-0000-4000-8000-000000000002';let tokenA:string,tokenB:string;
const settings=():Env=>({...env,KORTEX_ENV:'staging',PUBLIC_WEB_ORIGIN:web,PASSWORD_AUTH_ENABLED:'true',SESSION_HMAC_SECRET:'security-test-only-ephemeral-key-18'});
beforeAll(async()=>applyD1Migrations(env.DB,env.TEST_MIGRATIONS));
beforeEach(async()=>{tokenA=randomToken();tokenB=randomToken();for(const[id,token]of [[a,tokenA],[b,tokenB]]){await env.DB.prepare('DELETE FROM users WHERE id=?').bind(id).run();await env.DB.prepare('INSERT INTO users(id,created_at) VALUES(?,0)').bind(id).run();await env.DB.prepare('INSERT INTO sessions VALUES(?,?,?,?,NULL)').bind(await sessionHash(token,settings().SESSION_HMAC_SECRET!),id,Date.now(),Date.now()+60000).run()}});
async function call(path:string,method='GET',body?:unknown,headers:Record<string,string>={}){return proxy(new Request(web+'/api/v1/me/'+path,{method,headers:{Cookie:'__Host-kortex_session='+tokenA,Origin:web,'Content-Type':'application/json','X-CSRF-Token':await csrfToken(settings(),tokenA),'If-Match':'"preferences-0"',...headers},...(body===undefined?{}:{body:JSON.stringify(body)})}),undefined,web,{fetch:r=>handle(r,settings(),true)})}
it('rejects the browser CSRF mutation matrix without changing durable owner data',async()=>{
 const bad:Record<string,string>[]=[{Origin:'null'},{Origin:'https://evil.test'},{'Sec-Fetch-Site':'cross-site'},{'X-CSRF-Token':await csrfToken(settings(),tokenB)},{'X-CSRF-Token':'0'.repeat(64)},{'X-Kortex-Account':b},{Cookie:'__Host-kortex_session='+tokenA+'; __Host-kortex_session='+tokenB}];
 for(const headers of bad){const r=await call('preferences','PATCH',{sync_enabled:true},headers);expect([401,403]).toContain(r.status)}
 expect((await call('preferences')).status).toBe(200);expect((await(await call('preferences')).json() as any).sync_enabled).toBe(false);
});
it('error logs and exports omit credentials; CAS purge revokes only the owner and cannot replay',async()=>{
 const captured:string[]=[];const log=vi.spyOn(console,'error').mockImplementation((...v)=>captured.push(v.map(String).join(' ')));const marker='security-private-fixture-DO-NOT-LOG';
 try{const r=await call('preferences','PATCH',{sync_enabled:true,private_payload:marker});expect(r.status).toBe(400);expect(await r.text()).not.toContain(marker)}finally{log.mockRestore()}
 expect(captured.join('\n')).not.toContain(marker);expect(captured.join('\n')).not.toContain(tokenA);
 const snapshot=await call('export'),text=await snapshot.text();expect(snapshot.status).toBe(200);expect(text).toContain(a);for(const value of [b,tokenA,tokenB,'token_hash','password_hash'])expect(text).not.toContain(value);
 const deleted=await call('data','DELETE',{confirm:'delete-account'},{'If-Match':snapshot.headers.get('ETag')!});expect(deleted.status).toBe(204);expect((await call('export')).status).toBe(401);expect((await call('data','DELETE',{confirm:'delete-account'},{'If-Match':snapshot.headers.get('ETag')!})).status).toBe(401);expect(await env.DB.prepare('SELECT id FROM users WHERE id=?').bind(b).first('id')).toBe(b);expect(await env.DB.prepare('SELECT count(*) AS n FROM sessions WHERE user_id=?').bind(a).first('n')).toBe(0);
});
