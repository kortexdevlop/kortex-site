import {test,expect} from 'playwright/test';
import {writeFileSync} from 'node:fs';
test('live staging rejects forged sessions, account selectors and browser bearer confusion',async({request})=>{
 const web='https://kortex-web-staging.userpgl.workers.dev',api='https://kortex-api-staging.userpgl.workers.dev';const cases=[
  [web+'/api/v1/me',{'Cookie':'__Host-kortex_session='+'a'.repeat(43)}],
  [web+'/api/v1/me/library',{'Authorization':'Bearer forged','X-Kortex-Account':crypto.randomUUID()}],
  [web+'/api/v1/me/export?user_id='+crypto.randomUUID(),{}],
  [api+'/api/v1/me/library',{'Cookie':'__Host-kortex_session='+'a'.repeat(43),'X-Entry-Point':'AuthService'}],
 ] as const;const statuses=[];for(const[url,headers]of cases){const r=await request.get(url,{headers});expect([400,401,403,404]).toContain(r.status());expect(await r.text()).not.toMatch(/token_hash|password_hash|SESSION_HMAC_SECRET/);statuses.push(r.status())}
 const r=await request.get(web+'/workspace');const csp=r.headers()['content-security-policy'];expect(csp).toContain("frame-src https://kortex-run-staging.userpgl.workers.dev");expect(csp).not.toMatch(/script-src[^;]*unsafe-inline/);expect(r.headers()['cache-control']).toContain('no-store');writeFileSync('docs/evidence/security-auth-staging.json',JSON.stringify({date:new Date().toISOString(),passed:true,forged_requests:statuses,strict_csp:true,private_cache:true},null,2)+'\n');
});
