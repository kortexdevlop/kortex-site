import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {generateKeyPairSync,sign} from 'node:crypto';
import {certificateText,runtimeSigningText} from '../../shared/typescript/src/runtime.ts';
import {waitRuntimePublication} from '../../scripts/runtime-publication.mjs';
function fixture(){
 const f=JSON.parse(readFileSync('tests/fixtures/runtime-channels.signed-test.json')),pair=generateKeyPairSync('ed25519'),pub=pair.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64'),now=Date.now();
 const root={id:'propagation-test',public_key_base64:pub},keys={schema_version:1,version:1,issued_at:now-10000,expires:now+60000,root_key_id:root.id,keys:[{id:'release',public_key_base64:pub,not_before:now-10000,expires:now+60000,revoked:false}],signature:''};keys.signature=sign(null,certificateText(keys),pair.privateKey).toString('base64');
 const make=sequence=>{const document={...f.envelope.document,sequence,issued_at:now-1000,expires:now+30000,signing_key_id:'release',signature:''};document.signature=sign(null,runtimeSigningText(document),pair.privateKey).toString('base64');return{document,keys}};return{root,old:make(1),next:make(2)};
}
test('waits for valid older edge metadata and a transient unavailable edge before accepting exact publication',async()=>{
 const f=fixture(),responses=[Response.json(f.old),new Response('',{status:503}),Response.json(f.next)];let waits=0;
 const checked=await waitRuntimePublication('https://api.test',f.root,f.next.document,{fetcher:async()=>responses.shift(),delay:async()=>{waits++}});assert.equal(checked.signature,f.next.document.signature);assert.equal(waits,2);
});
test('propagation is bounded and altered or conflicting metadata is never admitted',async()=>{
 const f=fixture();await assert.rejects(waitRuntimePublication('https://api.test',f.root,f.next.document,{fetcher:async()=>Response.json(f.old),delay:async()=>{},attempts:2}),/PROPAGATION_TIMEOUT/);
 const altered=structuredClone(f.next);altered.document.expires++;await assert.rejects(waitRuntimePublication('https://api.test',f.root,f.next.document,{fetcher:async()=>Response.json(altered),delay:async()=>{}}),/RUNTIME_SIGNATURE/);
 await assert.rejects(waitRuntimePublication('https://api.test',f.root,{...f.next.document,signature:'different'},{fetcher:async()=>Response.json(f.next),delay:async()=>{}}),/RUNTIME_CHANNEL_MISMATCH/);
});
