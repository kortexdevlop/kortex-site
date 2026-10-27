import {ed25519} from '@noble/curves/ed25519.js';
import {artifact as validateArtifact} from './generated-validators';
import type {Descriptor} from './bridge';
export interface TrustKey {id:string;public_key_base64:string;not_before:number;expires:number;revoked:boolean}
export interface RootKey {id:string;public_key_base64:string}
export interface SignedKeys {schema_version:1;version:number;issued_at:number;expires:number;root_key_id:string;keys:TrustKey[];signature:string}
const id=(v:unknown):v is string=>typeof v==='string'&&/^[A-Za-z0-9_-]{1,64}$/.test(v);
const integer=(v:unknown):v is number=>typeof v==='number'&&Number.isSafeInteger(v)&&v>=0;
const object=(v:unknown):v is Record<string,unknown>=>v!==null&&typeof v==='object'&&!Array.isArray(v);
const exact=(v:unknown,keys:string[])=>object(v)&&Object.keys(v).sort().join('|')===keys.sort().join('|');
function decode(value:string,length:number):Uint8Array<ArrayBuffer> {try{if(typeof value!=='string'||value.length!==Math.ceil(length/3)*4)throw Error();const b=Uint8Array.from(atob(value),c=>c.charCodeAt(0));if(b.length!==length||btoa(String.fromCharCode(...b))!==value)throw Error();return b}catch{throw Error('INTEGRITY_ERROR')}}
function validKey(k:TrustKey){if(!exact(k,['id','public_key_base64','not_before','expires','revoked'])||!id(k.id)||!integer(k.not_before)||!integer(k.expires)||k.expires<=k.not_before||typeof k.revoked!=='boolean')throw Error('INVALID_TRUST_STORE');decode(k.public_key_base64,32)}
export function artifactSigningText(proof:Descriptor):Uint8Array<ArrayBuffer> {
 if(!validateArtifact(proof))throw Error('INVALID_DESCRIPTOR');decode(proof.signature,64);
 return new TextEncoder().encode(`kortex-artifact-v1\n${proof.tool_id}\n${proof.version}\n${proof.abi_version}\n${proof.sha256}\n`);
}
export function keysSigningText(document:SignedKeys):Uint8Array<ArrayBuffer> {
 if(!exact(document,['schema_version','version','issued_at','expires','root_key_id','keys','signature'])||document.schema_version!==1||!integer(document.version)||document.version<1||!integer(document.issued_at)||!integer(document.expires)||document.issued_at>=document.expires||!id(document.root_key_id)||!Array.isArray(document.keys)||document.keys.length>256)throw Error('INVALID_TRUST_STORE');
 const keys=[...document.keys];for(const k of keys)validKey(k);keys.sort((a,b)=>a.id<b.id?-1:a.id>b.id?1:0);let previous='';let text=`kortex-keys-v1\n${document.version}\n${document.issued_at}\n${document.expires}\n${document.root_key_id}\n`;
 for(const k of keys){if(k.id===previous)throw Error('INVALID_TRUST_STORE');previous=k.id;text+=`${k.id}|${k.public_key_base64}|${k.not_before}|${k.expires}|${k.revoked}\n`;}
 return new TextEncoder().encode(text);
}
/** Host-owned trust snapshot. The root and minimum accepted version come from the host. */
export class TrustStore {
 #keys:Map<string,TrustKey>;#expires=Number.MAX_SAFE_INTEGER;
 constructor(keys:readonly TrustKey[]){this.#keys=new Map();if(keys.length>256)throw Error('INVALID_TRUST_STORE');for(const key of keys){validKey(key);if(this.#keys.has(key.id))throw Error('INVALID_TRUST_STORE');this.#keys.set(key.id,{...key});}}
 static fromSignedKeys(document:SignedKeys,root:RootKey,minimumVersion:number,now=Date.now()):TrustStore {
  const text=keysSigningText(document);if(!integer(minimumVersion)||!integer(now))throw Error('INVALID_TRUST_STORE');if(document.version<minimumVersion)throw Error('TRUST_ROLLBACK');
  if(!exact(root,['id','public_key_base64'])||!id(root.id)||root.id!==document.root_key_id||now<document.issued_at||now>=document.expires)throw Error('UNTRUSTED_KEY');
  if(!ed25519.verify(decode(document.signature,64),text,decode(root.public_key_base64,32),{zip215:false}))throw Error('INTEGRITY_ERROR');
  const store=new TrustStore(document.keys);store.#expires=document.expires;return store;
 }
 revoke(id:string){const key=this.#keys.get(id);if(key)key.revoked=true;}
 async verify(bytes:Uint8Array,proof:Descriptor,now=Date.now()):Promise<void>{
  const started=Date.now();if(bytes.length===0||bytes.length>25*1024*1024)throw Error('SIZE_LIMIT');if(proof.abi_version!=='1.0')throw Error('UNSUPPORTED_ABI');
  proof=structuredClone(proof);bytes=bytes.slice();const message=artifactSigningText(proof);
  const digest=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes as BufferSource)),b=>b.toString(16).padStart(2,'0')).join('');if(digest!==proof.sha256)throw Error('INTEGRITY_ERROR');
  const observed=now+Math.max(0,Date.now()-started);const key=this.#keys.get(proof.signing_key_id);if(!integer(observed)||observed>=this.#expires||!key||key.revoked||observed<key.not_before||observed>=key.expires)throw Error('UNTRUSTED_KEY');
  if(!ed25519.verify(decode(proof.signature,64),message,decode(key.public_key_base64,32),{zip215:false}))throw Error('INTEGRITY_ERROR');
 }
}
