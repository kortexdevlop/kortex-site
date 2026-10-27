const {test}=require('node:test'),assert=require('node:assert/strict'),fs=require('node:fs'),crypto=require('node:crypto'),path=require('node:path');
const root=path.resolve(__dirname,'../..');const read=p=>fs.readFileSync(path.join(root,p));const digest=b=>crypto.createHash('sha256').update(b).digest('hex');
test('frozen calculator package, module and wire corpus have recorded hashes',()=>{
 const ref=JSON.parse(read('tests/golden/calculatrice-reference.json'));const fixture=JSON.parse(read('target/calculatrice-fixture/fixture.json'));
 assert.equal(digest(read('target/calculatrice-fixture/calculatrice.tool')),ref.archive_sha256);
 assert.equal(digest(read('target/calculatrice-fixture/module.wasm')),ref.guest_sha256);
 assert.equal(digest(read('tests/golden/calculatrice.ndjson')),ref.goldens_sha256);
 assert.equal(fixture.descriptor.sha256,ref.archive_sha256);assert.equal(fixture.guest_sha256,ref.guest_sha256);
 const key=crypto.createPublicKey({key:Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(fixture.trust[fixture.descriptor.signing_key_id],'base64')]),format:'der',type:'spki'});
 const message=Buffer.from(`kortex-artifact-v1\n${ref.tool_id}\n${ref.version}\n${ref.abi_version}\n${ref.archive_sha256}\n`);
 assert.ok(crypto.verify(null,message,key,Buffer.from(fixture.descriptor.signature,'base64')));
});
