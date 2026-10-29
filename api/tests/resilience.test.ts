import {env} from 'cloudflare:workers';
import {beforeAll,it,expect} from 'vitest';
import {handle} from '../src/index';
import {seedArtifact} from './helpers';
import {sessionHash} from '../src/services/identity-store';
import {csrfToken} from '../src/security/csrf';
import {MODEL} from '../src/services/ai-config';
import {randomToken} from '../src/security/sessions';
const tool='resilience-test',user='20400000-0000-4000-8000-000000000001',token=randomToken(),secret='resilience-test-session-key-32-characters',web='http://localhost:4321';
let artifact:Awaited<ReturnType<typeof seedArtifact>>;
beforeAll(async()=>{artifact=await seedArtifact(tool,new Uint8Array([1,2,3,4]));await env.DB.prepare('INSERT INTO users(id,created_at) VALUES(?,0)').bind(user).run();await env.DB.prepare('INSERT INTO sessions VALUES(?,?,?,?,NULL)').bind(await sessionHash(token,secret),user,Date.now(),Date.now()+3600000).run()});
const request=(path:string)=>new Request('http://api.test/api/v1/'+path);
it('a D1 outage returns a bounded error, preserves data and recovers without a code restart',async()=>{
 const broken={...env,DB:{prepare(){throw Error('PRIVATE_database_details')},batch(){throw Error('PRIVATE_database_details')}}};
 const failed=await handle(request('tools/'+tool),broken);expect(failed.status).toBe(503);expect(await failed.text()).not.toContain('PRIVATE');expect((await handle(request('health'),broken)).status).toBe(200);
 expect((await handle(request('tools/'+tool),env)).status).toBe(200);expect((await env.DB.prepare('SELECT sha256 FROM artifacts WHERE tool_id=?').bind(tool).first())?.sha256).toBe(artifact.sha256);
});
it('R2 failure cannot be mistaken for a successful download and recovery keeps exact bytes',async()=>{
 const broken={...env,TOOLS_BUCKET:{head:env.TOOLS_BUCKET.head.bind(env.TOOLS_BUCKET),put:env.TOOLS_BUCKET.put.bind(env.TOOLS_BUCKET),async get(){throw Error('PRIVATE object name')}}};
 const r=await handle(request('downloads/'+tool+'/1.0.0'),broken);expect(r.status).toBe(503);expect(await r.text()).not.toContain('PRIVATE');expect((await handle(request('tools/'+tool),broken)).status).toBe(200);
 const restored=await handle(request('downloads/'+tool+'/1.0.0'),env);expect(restored.status).toBe(200);expect([...new Uint8Array(await restored.arrayBuffer())]).toEqual([1,2,3,4]);
});
it('an expired session cannot read or mutate account state, and no header re-enables it',async()=>{
 const expired=randomToken();await env.DB.prepare('INSERT INTO sessions VALUES(?,?,?,?,NULL)').bind(await sessionHash(expired,secret),user,0,1).run();
 const settings={...env,PASSWORD_AUTH_ENABLED:'true',SESSION_HMAC_SECRET:secret};
 const r=await handle(new Request(web+'/api/v1/me/preferences',{headers:{Cookie:'__Host-kortex_session='+expired,Origin:web,'X-Kortex-Account':user}}),settings,true);expect(r.status).toBe(401);expect((await env.DB.prepare('SELECT count(*) n FROM users WHERE id=?').bind(user).first())?.n).toBe(1);
});
it('AI provider outage or invalid output fails closed while manual tools remain available',async()=>{
 const settings={...env,PASSWORD_AUTH_ENABLED:'true',SESSION_HMAC_SECRET:secret,AI_ENABLED:'true',AI_MODEL:MODEL,AI_USER_DAILY_REQUESTS:'50',AI_GLOBAL_DAILY_REQUESTS:'100'};
 const body={message:'Propose a calculation',available_tool_ids:[tool],consent_context:{ai_allowed:true,contains_private_data:false}};
 for(const run of [async()=>{throw Error('PRIVATE provider prompt')},async()=>({response:{v:1,steps:[{tool_id:tool,action_id:'execute-shell',input:{},requires_consent:false}],summary:'unsafe'}})]){
  const r=await handle(new Request(web+'/api/v1/ai/plan',{method:'POST',headers:{Cookie:'__Host-kortex_session='+token,Origin:web,'Content-Type':'application/json','X-CSRF-Token':await csrfToken(settings,token)},body:JSON.stringify(body)}),{...settings,AI:{run}},true);expect(r.status).toBe(503);expect(await r.text()).not.toContain('PRIVATE');expect((await handle(request('tools/'+tool),env)).status).toBe(200);
 }
});
it('malicious catalogue selectors are rejected without writes or arbitrary error disclosure',async()=>{
 const before=await env.DB.prepare('SELECT count(*) n FROM tools').first();
 for(const suffix of ['?limit=999999999','?q=x&unknown=DROP%20TABLE%20tools','?cursor=not-a-signed-cursor'])expect((await handle(request('tools'+suffix),env)).status).toBe(400);
 expect(await env.DB.prepare('SELECT count(*) n FROM tools').first()).toEqual(before);
});
