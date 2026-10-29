import {env} from 'cloudflare:workers';
import {describe,it,expect} from 'vitest';
import {seedArtifact} from './helpers';
import {revocations} from '../src/routes/revocations';
import {revocationText,type RevocationDocument} from '../../shared/typescript/src/revocation';
import type {Env} from '../src/env';
describe('offline-root signed revocation endpoint',()=>{
 it('checks real signatures, TTL and D1 coverage before serving a snapshot',async()=>{
  const artifact=await seedArtifact('revocation-coverage',new Uint8Array([1,2,3]));
  const key=await crypto.subtle.generateKey('Ed25519',true,['sign','verify']) as CryptoKeyPair;
  const b64=(b:ArrayBuffer)=>btoa(String.fromCharCode(...new Uint8Array(b)));
  const root={id:'test-revocation-root',public_key_base64:b64(await crypto.subtle.exportKey('raw',key.publicKey) as ArrayBuffer)};
  const now=Date.now();const d:RevocationDocument={schema_version:1,version:1,issued_at:now-1000,valid_until:now+86399000,root_key_id:root.id,revoked_keys:[],revoked_artifacts:[],signature:''};
  const sign=async()=>{d.signature=b64(await crypto.subtle.sign('Ed25519',key.privateKey,revocationText(d)))};
  const call=async(method='GET')=>revocations(new Request('https://api.test/api/v1/trust/revocations',{method}),{...env,REVOCATION_ROOT:JSON.stringify(root),REVOCATION_DOCUMENT:JSON.stringify(d)} as Env,'test');
  await sign();expect((await call())!.status).toBe(200);expect(await(await call('HEAD'))!.text()).toBe('');
  d.version=2;expect((await call())!.status).toBe(503);await sign();
  await env.DB.prepare("UPDATE tools SET status='archived',published_version=NULL WHERE id='revocation-coverage'").run();
  await env.DB.prepare('INSERT INTO revocations(request_id,tool_id,version,reason,created_at) VALUES(?,?,?,?,?)').bind(crypto.randomUUID(),'revocation-coverage','1.0.0','test',now).run();
  expect((await call())!.status).toBe(503);
  d.revoked_artifacts=[{tool_id:'revocation-coverage',version:'1.0.0',sha256:artifact.sha256}];await sign();expect((await call())!.status).toBe(200);
  d.issued_at=now-86400001;d.valid_until=now-1;await sign();expect((await call())!.status).toBe(503);
 });
});
