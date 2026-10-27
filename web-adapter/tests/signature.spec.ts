import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {generateKeyPairSync,sign} from 'node:crypto';
import {serve,root} from '../../tests/web/harness';
const load=(p:string)=>JSON.parse(readFileSync(resolve(root,p),'utf8'));
const proof=load('tests/fixtures/artifact-signed-test.json'),anchor=load('shared/fixtures/trust-test-root.json'),document=load('shared/fixtures/trust-test-keys.json');
const bytes=Array.from(readFileSync(resolve(root,'tests/fixtures/artifact-signed-test.tool')));
let server:Awaited<ReturnType<typeof serve>>;test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>server.close());
test('signature B.2 commune Rust/Web : paquet canonique, mutations et rotation signée',async({page})=>{
 const pair=generateKeyPairSync('ed25519');const rotationRoot={id:'rotation-test-root',public_key_base64:pair.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64')};
 const rotations=[[],document.keys.map((k:any)=>({...k,revoked:true}))].map(keys=>{
  const d={schema_version:1,version:2,issued_at:0,expires:4102444800000,root_key_id:rotationRoot.id,keys,signature:''};
  const text=`kortex-keys-v1\n2\n0\n4102444800000\n${d.root_key_id}\n`+keys.map((k:any)=>`${k.id}|${k.public_key_base64}|${k.not_before}|${k.expires}|${k.revoked}\n`).join('');d.signature=sign(null,Buffer.from(text),pair.privateKey).toString('base64');return d;
 });
 await page.goto(server.origin);
 const result=await page.evaluate(async({proof,anchor,document,bytes,rotations,rotationRoot})=>{
  const api=await import('/adapter/interop.js' as string);const trustModule=await import('/adapter/trust.js' as string);const raw=new Uint8Array(bytes);
  const store=api.TrustStore.fromSignedKeys(document,anchor,1,1000);await store.verify(raw,proof,1000);
  const text=new TextDecoder().decode(trustModule.artifactSigningText(proof));let rejected=0;
  const corrupted=raw.slice();corrupted[40]^=1;try{await store.verify(corrupted,proof,1000)}catch{rejected++}
  for(const p of [{...proof,version:'2.0.0'},{...proof,abi_version:'2.0'},{...proof,tool_id:'other-tool'},{...proof,signature:proof.signature.replace(/^./,proof.signature[0]==='A'?'B':'A')},{...proof,signing_key_id:'retired-key'},{...proof,schema_version:2},{...proof,extra:1}]){try{await store.verify(raw,p,1000)}catch{rejected++}}
  for(const doc of rotations){const removed=api.TrustStore.fromSignedKeys(doc,rotationRoot,2,1000);try{await removed.verify(raw,proof,1000)}catch{rejected++}}
  try{api.TrustStore.fromSignedKeys(document,anchor,2,1000)}catch{rejected++}
  try{api.TrustStore.fromSignedKeys({...document,version:2},anchor,1,1000)}catch{rejected++}
  const tool=await api.VerifiedTool.verifyArchive(raw,proof,store);const bridge=await api.startVerifiedTool(tool);try{
   await bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind:'init',payload:{}});
   const action=await bridge.dispatch({v:1,request_id:crypto.randomUUID(),kind:'action.invoke',payload:{action_id:'multiply',input:{a:25,b:48}}});
   return {rejected,text,result:action.data.result};
  }finally{bridge.stop()}
 },{proof,anchor,document,bytes,rotations,rotationRoot});
 expect(result.rejected).toBe(12);expect(result.result).toBe(1200);expect(result.text).toBe(`kortex-artifact-v1\n${proof.tool_id}\n${proof.version}\n${proof.abi_version}\n${proof.sha256}\n`);
});
