// Test-only packager; ephemeral signing key is never persisted.
const fs=require('node:fs'),path=require('node:path'),crypto=require('node:crypto');
const {makeTool}=require('./make-tool.cjs');const root=path.resolve(__dirname,'../..'),dir=process.env.CALCULATRICE_FIXTURE_DIR||path.join(root,'target/calculatrice-fixture');
function fixture(){return JSON.parse(fs.readFileSync(path.join(dir,'fixture.json'),'utf8'))}
module.exports={fixture,dir};
if(require.main===module){
 const wasmPath=process.env.CALCULATRICE_WASM||path.join(root,'tests/fixtures/calculatrice-reference.wasm');
 const f=makeTool({wasmPath,manifestPath:path.join(root,'tests/fixtures/calculatrice-v1-manifest.json'),contractPath:path.join(root,'tools/calculatrice/contract.json'),iconPath:path.join(root,'tools/calculatrice/assets/icon.svg')});
 fs.mkdirSync(dir,{recursive:true});if(fs.existsSync(path.join(dir,'fixture.json'))&&fixture().descriptor.sha256===f.descriptor.sha256&&fixture().keys.every(k=>k.expires>Date.now()+3600000)){console.log(JSON.stringify(fixture(),null,2));process.exit(0)}fs.writeFileSync(path.join(dir,'calculatrice.tool'),f.bytes);fs.writeFileSync(path.join(dir,'calculatrice.tool.json'),JSON.stringify(f.descriptor));fs.writeFileSync(path.join(dir,'trust.json'),JSON.stringify(f.trust));
 fs.writeFileSync(path.join(dir,'module.wasm'),fs.readFileSync(wasmPath));fs.writeFileSync(path.join(dir,'fixture.json'),JSON.stringify({descriptor:f.descriptor,trust:f.trust,keys:f.keys,guest_sha256:crypto.createHash('sha256').update(fs.readFileSync(wasmPath)).digest('hex')}));console.log(JSON.stringify(fixture(),null,2));
}
