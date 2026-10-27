import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'../..');
const bytes=p=>readFileSync(resolve(root,p));
const json=p=>JSON.parse(bytes(p));
const sha=b=>createHash('sha256').update(b).digest('hex');
test('six frozen schemas and PACKAGE_FORMAT_V1 reference remain unchanged',()=>{
 const rows=[...bytes('docs/CONTRACTS.md').toString().matchAll(/\| (shared\/schemas\/[^ ]+) \| ([a-f0-9]{64}) \|/g)];
 assert.equal(rows.length,6);for(const [,path,digest] of rows)assert.equal(sha(bytes(path)),digest,path);
 const g=json('tests/golden/calculatrice-artifact.json'),archive=bytes(g.archive),proof=json(g.descriptor);
 assert.equal(archive.length,g.archive_bytes);assert.equal(sha(archive),g.sha256);assert.equal(proof.sha256,g.sha256);
 assert.equal(sha(bytes('shared/schemas/artifact.schema.json')),g.artifact_schema_sha256);
 const files=new Map();let at=0;while(archive.readUInt32LE(at)===0x04034b50){
  assert.equal(archive.readUInt16LE(at+8),0);assert.equal(archive.readUInt16LE(at+10),0);assert.equal(archive.readUInt16LE(at+12),33);
  const n=archive.readUInt16LE(at+26),extra=archive.readUInt16LE(at+28),size=archive.readUInt32LE(at+18);assert.equal(extra,0);
  const name=archive.subarray(at+30,at+30+n).toString();files.set(name,archive.subarray(at+30+n,at+30+n+size));at+=30+n+size;
 }
 assert.deepEqual([...files.keys()],[...files.keys()].sort());assert.equal(sha(files.get('module.wasm')),g.guest_sha256);
 const req=json('shared/fixtures/publication-request.json');assert.deepEqual(req.descriptor,proof);assert.deepEqual(req.manifest,JSON.parse(files.get('manifest.json')));assert.deepEqual(req.contract,JSON.parse(files.get('contract.json')));
});
