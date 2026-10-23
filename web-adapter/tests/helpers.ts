import {generateKeyPairSync,sign} from 'node:crypto';
import type {Page} from 'playwright/test';
import {readFileSync} from 'node:fs';
import {root} from '../../tests/web/harness';
import {resolve} from 'node:path';
export const manifest=JSON.parse(readFileSync(resolve(root,'shared/fixtures/manifest.valid.json'),'utf8'));
export const contract=JSON.parse(readFileSync(resolve(root,'shared/fixtures/contract.valid.json'),'utf8'));contract.actions[0].id='multiply';
export async function prepare(page:Page,permissions:string[]=[],wasmURL='/fixtures/guest.wasm'){
 const pair=generateKeyPairSync('ed25519');const publicKey=Array.from(pair.publicKey.export({format:'der',type:'spki'}).subarray(-32));
 await page.exposeFunction('signTestArtifact',(message:number[])=>sign(null,Buffer.from(message),pair.privateKey).toString('base64'));
 await page.evaluate(async({manifest,contract,publicKey,wasmURL})=>{
  const api=await import('/adapter/host.js' as string);
  const raw=new Uint8Array(await(await fetch(wasmURL)).arrayBuffer());
  const leb=(n:number)=>{const b=[];do{let v=n&127;n=Math.floor(n/128);if(n)v|=128;b.push(v);}while(n);return b;};
  const section=(name:string,value:unknown)=>{const n=new TextEncoder().encode(name),v=new TextEncoder().encode(JSON.stringify(value)),payload=[...leb(n.length),...n,...v];return [0,...leb(payload.length),...payload]};
  const bytes=new Uint8Array([...raw,...section('kortex.manifest',manifest),...section('kortex.contract',contract)]);
  const pub=new Uint8Array(publicKey);
  async function admit(bytes:Uint8Array,metadata=manifest,contracts=contract){
   const sha256=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),b=>b.toString(16).padStart(2,'0')).join('');
   const text=new TextEncoder().encode(`kortex-artifact-v1\n${metadata.id}\n${metadata.version}\n1.0\n${sha256}\n`);
   const signature=await (window as any).signTestArtifact(Array.from(text));
   return api.VerifiedTool.verifySignedModule(bytes,metadata,contracts,{tool_id:metadata.id,version:metadata.version,abi_version:'1.0',sha256,signing_key_id:'ephemeral-test',signature},{'ephemeral-test':pub});
  }
  const tool=await admit(bytes);(window as any).adapterTest={api,bytes,tool,admit,manifest,contract,section,publicKey:pub};
 },{manifest:{...manifest,permissions},contract,publicKey,wasmURL});
}
