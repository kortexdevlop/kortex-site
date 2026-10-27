import {env,exports} from 'cloudflare:workers';
import {describe,it,expect} from 'vitest';
import {handle} from '../src/index';
import {validateEnv,type Env as ApiEnv} from '../src/env';
declare global {namespace Cloudflare {interface Env extends ApiEnv {} interface GlobalProps {mainModule:typeof import('../src/index')}}}
const id='92000000-0000-4000-8000-000000000001';
describe('API V1 in real workerd, local bindings',()=>{
 it('GET health returns the exact public payload and security headers',async()=>{
  const response=await exports.default.fetch(new Request('https://api.test/api/v1/health',{headers:{'X-Request-ID':id}}));
  expect(response.status).toBe(200);expect(await response.json()).toEqual({status:'ok',api_version:'1'});
  expect(response.headers.get('x-request-id')).toBe(id);expect(response.headers.get('cache-control')).toBe('no-store');
  expect(response.headers.get('content-security-policy')).toContain("default-src 'none'");expect(response.headers.get('x-content-type-options')).toBe('nosniff');
 });
 it('unknown route is a bounded JSON 404 with a generated request ID',async()=>{
  const response=await exports.default.fetch(new Request('https://api.test/api/v1/unknown',{headers:{'X-Request-ID':'untrusted-value'}}));
  const body=await response.json() as {error:{code:string;request_id:string}};
  expect(response.status).toBe(404);expect(body.error.code).toBe('NOT_FOUND');expect(body.error.request_id).toMatch(/^[0-9a-f-]{36}$/);expect(body.error.request_id).not.toBe('untrusted-value');
 });
 it('HEAD has no body, unsupported health method has Allow and JSON error',async()=>{
  const head=await exports.default.fetch(new Request('https://api.test/api/v1/health',{method:'HEAD'}));expect(head.status).toBe(200);expect(await head.text()).toBe('');
  const post=await exports.default.fetch(new Request('https://api.test/api/v1/health',{method:'POST'}));expect(post.status).toBe(405);expect(post.headers.get('allow')).toContain('GET');
 });
 it('CORS allows the configured origin only and validates preflight',async()=>{
  const allowed=await exports.default.fetch(new Request('https://api.test/api/v1/health',{headers:{Origin:env.PUBLIC_WEB_ORIGIN}}));expect(allowed.headers.get('access-control-allow-origin')).toBe(env.PUBLIC_WEB_ORIGIN);
  const denied=await exports.default.fetch(new Request('https://api.test/api/v1/health',{headers:{Origin:'https://untrusted.test'}}));expect(denied.status).toBe(403);expect(denied.headers.has('access-control-allow-origin')).toBe(false);
  const preflight=await exports.default.fetch(new Request('https://api.test/api/v1/health',{method:'OPTIONS',headers:{Origin:env.PUBLIC_WEB_ORIGIN,'Access-Control-Request-Method':'GET','Access-Control-Request-Headers':'X-Request-ID'}}));expect(preflight.status).toBe(204);
  const bad=await exports.default.fetch(new Request('https://api.test/api/v1/health',{method:'OPTIONS',headers:{Origin:env.PUBLIC_WEB_ORIGIN,'Access-Control-Request-Method':'DELETE'}}));expect(bad.status).toBe(400);
 });
 it('configuration/secrets/binding errors are never returned in HTTP',async()=>{
  const response=await handle(new Request('https://api.test/api/v1/health'),{...env,KORTEX_ENV:'production',PUBLIC_WEB_ORIGIN:'https://production.invalid',OIDC_CLIENT_SECRET:'TEST_SENTINEL_DO_NOT_RETURN'});
  expect(response.status).toBe(503);const text=await response.text();expect(text).not.toContain('TEST_SENTINEL');expect(text).not.toContain('production.invalid');expect(text).not.toContain('DB');
  expect(validateEnv({...env,DB:undefined})).toBeNull();expect(validateEnv({...env,TOOLS_BUCKET:undefined})).toBeNull();expect(validateEnv({...env,PUBLIC_WEB_ORIGIN:'http://localhost:4321/'})).toBeNull();
 });
 it('D1 and R2 are actual local runtime bindings and D1 batch rolls back on failure',async()=>{
  await env.DB.prepare('CREATE TABLE api_probe(id INTEGER PRIMARY KEY)').run();
  await expect(env.DB.batch([env.DB.prepare('INSERT INTO api_probe VALUES(1)'),env.DB.prepare('INSERT INTO api_probe VALUES(1)')])).rejects.toThrow();
  expect(await env.DB.prepare('SELECT count(*) AS n FROM api_probe').first('n')).toBe(0);
  await env.TOOLS_BUCKET.put('api-test-only/probe','local binding');const object=await env.TOOLS_BUCKET.get('api-test-only/probe');expect(await object!.text()).toBe('local binding');await env.TOOLS_BUCKET.delete('api-test-only/probe');
 });
});
