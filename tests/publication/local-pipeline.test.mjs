import test from 'node:test';import assert from 'node:assert/strict';
import {mkdtempSync,readFileSync,writeFileSync,mkdirSync} from 'node:fs';import {resolve} from 'node:path';import {createHash,createPublicKey,verify} from 'node:crypto';
import {prepareRelease,validatePublication} from '../../scripts/release-tool.mjs';
const root=resolve(import.meta.dirname,'../..');mkdirSync(resolve(root,'target'),{recursive:true});const dir=mkdtempSync(resolve(root,'target/publication-test-'));
const read=p=>JSON.parse(readFileSync(p));
test('offline staging: reproducible bytes, signed payload, idempotence and immutable version',()=>{
 const old=globalThis.fetch;globalThis.fetch=()=>{throw Error('NETWORK_FORBIDDEN')};try{
  const a=prepareRelease({tool:'calculatrice',dryRun:true,out:resolve(dir,'a')});const b=prepareRelease({tool:'calculatrice',dryRun:true,out:resolve(dir,'b')});assert.equal(a.sha256,b.sha256);assert.deepEqual(readFileSync(a.archive),readFileSync(b.archive));assert.equal(a.remote_writes,0);assert.equal(a.cargo_offline,true);assert.deepEqual(a.calls,['build','validate','package','hash','sign','verify']);
  const again=prepareRelease({tool:'calculatrice',dryRun:true,out:resolve(dir,'a')});assert.equal(again.idempotent,true);assert.ok(!again.calls.includes('sign'));assert.deepEqual(read(again.publication_request),read(a.publication_request));
  const req=read(a.publication_request);validatePublication(req);assert.equal(req.r2_key,`tools/calculatrice/1.0.0/${a.sha256}.tool`);assert.equal(a.sha256,read(resolve(root,'tests/golden/calculatrice-artifact.json')).sha256);
  const bundle=read(a.host_trust);const key=bundle.document.keys[0];const publicKey=createPublicKey({key:Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(key.public_key_base64,'base64')]),format:'der',type:'spki'});
  const text=`kortex-artifact-v1\n${req.descriptor.tool_id}\n${req.descriptor.version}\n${req.descriptor.abi_version}\n${req.descriptor.sha256}\n`;assert.ok(verify(null,Buffer.from(text),publicKey,Buffer.from(req.descriptor.signature,'base64')));assert.equal(createHash('sha256').update(readFileSync(a.archive)).digest('hex'),req.descriptor.sha256);
  assert.throws(()=>validatePublication({...req,request_id:'invalid'}));assert.throws(()=>validatePublication({...req,r2_key:'overwrite.tool'}));assert.throws(()=>validatePublication({...req,manifest:{...req.manifest,version:'2.0.0'}}));assert.throws(()=>validatePublication({...req,descriptor:{...req.descriptor,signature:'fake'}}));assert.throws(()=>validatePublication({...req,contract:{...req.contract,extra:1}}));
  assert.throws(()=>prepareRelease({tool:'calculatrice',dryRun:false,out:resolve(dir,'a')}));req.descriptor.sha256='0'.repeat(64);writeFileSync(a.publication_request,JSON.stringify(req));assert.throws(()=>prepareRelease({tool:'calculatrice',dryRun:true,out:resolve(dir,'a')}));
 }finally{globalThis.fetch=old}
});
