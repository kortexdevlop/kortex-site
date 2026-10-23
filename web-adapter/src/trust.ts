import {ed25519} from '@noble/curves/ed25519.js';
import type {Descriptor} from './bridge';
/** Host-owned trust store. Unknown, expired and explicitly revoked keys fail closed. */
export interface TrustKey {id:string;public_key_base64:string;not_before:number;expires:number;revoked:boolean}
export class TrustStore {
 #keys:Map<string,TrustKey>;
 constructor(keys:readonly TrustKey[]){this.#keys=new Map();for(const key of keys){if(this.#keys.has(key.id)||!key.id||!Number.isFinite(key.not_before)||!Number.isFinite(key.expires)||key.expires<=key.not_before)throw Error('INVALID_TRUST_STORE');const bytes=decode(key.public_key_base64);if(bytes.length!==32)throw Error('INVALID_TRUST_STORE');this.#keys.set(key.id,{...key});}}
 revoke(id:string){const key=this.#keys.get(id);if(key)key.revoked=true;}
 async verify(bytes:Uint8Array,proof:Descriptor,now=Date.now()):Promise<void>{
  if(bytes.length===0||bytes.length>25*1024*1024)throw Error('SIZE_LIMIT');
  if(proof.abi_version!=='1.0')throw Error('UNSUPPORTED_ABI');
  if(!/^[a-f0-9]{64}$/.test(proof.sha256)||/[\r\n]/.test(proof.tool_id+proof.version))throw Error('INTEGRITY_ERROR');
  const key=this.#keys.get(proof.signing_key_id);if(!key||key.revoked||now<key.not_before||now>=key.expires)throw Error('UNTRUSTED_KEY');
  const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes as BufferSource)),b=>b.toString(16).padStart(2,'0')).join('');if(digest!==proof.sha256)throw Error('INTEGRITY_ERROR');
  const signature=decode(proof.signature),publicKey=decode(key.public_key_base64);if(signature.length!==64)throw Error('INTEGRITY_ERROR');
  const message=new TextEncoder().encode(`kortex-artifact-v1\n${proof.tool_id}\n${proof.version}\n1.0\n${digest}\n`);
  if(!ed25519.verify(signature,message,publicKey,{zip215:false}))throw Error('INTEGRITY_ERROR');if(key.revoked||now>=key.expires)throw Error('UNTRUSTED_KEY');
 }
}
function decode(value:string):Uint8Array {try{return Uint8Array.from(atob(value),c=>c.charCodeAt(0))}catch{throw Error('INTEGRITY_ERROR')}}
