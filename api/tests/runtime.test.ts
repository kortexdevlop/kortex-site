import {env} from 'cloudflare:workers';
import {applyD1Migrations} from 'cloudflare:test';
import {it,expect} from 'vitest';
import {runtime} from '../src/routes/runtime';
import {runtimeSigningText,certificateText,type RuntimeDocument,type KeyCertificate} from '../../shared/typescript/src/runtime';
import type {Env} from '../src/env';
it('serves real signed runtime channels and refuses forged, expired or revoked publisher metadata',async()=>{
 await applyD1Migrations(env.DB,env.TEST_MIGRATIONS);const now=Date.now(),rootKey=await crypto.subtle.generateKey('Ed25519',true,['sign','verify']) as CryptoKeyPair,key=await crypto.subtle.generateKey('Ed25519',true,['sign','verify']) as CryptoKeyPair,b64=(b:ArrayBuffer)=>btoa(String.fromCharCode(...new Uint8Array(b)));
 const root={id:'runtime-api-test-root',public_key_base64:b64(await crypto.subtle.exportKey('raw',rootKey.publicKey) as ArrayBuffer)},publicKey=b64(await crypto.subtle.exportKey('raw',key.publicKey) as ArrayBuffer);
 const keys:KeyCertificate={schema_version:1,version:1,issued_at:now-1000,expires:now+86400000,root_key_id:root.id,keys:[{id:'runtime-api-test-release',public_key_base64:publicKey,not_before:now-1000,expires:now+86400000,revoked:false}],signature:''};keys.signature=b64(await crypto.subtle.sign('Ed25519',rootKey.privateKey,certificateText(keys)));
 const document:RuntimeDocument={schema_version:1,sequence:1,issued_at:now-1000,expires:now+3600000,channels:{stable:null,staging:{runtime_version:'1.0.1',abi_versions:['1.0'],protocol_versions:[1],ui_schema_versions:['1.0'],web:{origin:'https://run.example.test',version_url:'https://run.example.test/version.json',bundle_sha256:'a'.repeat(64),script_integrity:'sha256-'+btoa(String.fromCharCode(...new Uint8Array(32)))},desktop:null}},signing_key_id:keys.keys[0].id,signature:''};
 const sign=async()=>{document.signature=b64(await crypto.subtle.sign('Ed25519',key.privateKey,runtimeSigningText(document)))};await sign();
 await env.DB.prepare('INSERT INTO signing_keys(id,public_key_base64,trust_version,not_before,expires,revoked) VALUES(?,?,1,?,?,0)').bind(keys.keys[0].id,publicKey,now-1000,now+86400000).run();
 const call=async(method='GET')=>runtime(new Request('https://api.test/api/v1/runtime/channels',{method}),{...env,RUNTIME_ROOT:JSON.stringify(root),RUNTIME_CHANNELS:JSON.stringify({document,keys})} as Env,'test');
 expect((await call())!.status).toBe(200);expect(await(await call('HEAD'))!.text()).toBe('');document.sequence=2;expect((await call())!.status).toBe(503);await sign();expect((await call())!.status).toBe(200);
 await env.DB.prepare('INSERT INTO revocations(request_id,signing_key_id,reason,created_at) VALUES(?,?,?,?)').bind(crypto.randomUUID(),keys.keys[0].id,'test compromised publisher',Date.now()).run();expect((await call())!.status).toBe(503);
});
