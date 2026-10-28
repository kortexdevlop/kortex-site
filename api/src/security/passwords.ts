import {scrypt} from 'node:crypto';
import type {Env} from '../env';
import {b64,encoder} from './sessions';
import {base64url} from './admin-auth';
import {equal} from './csrf';
// OWASP scrypt profile: N=2^15, r=8, p=3 (32 MiB). Never take work factors from input/DB.
const COST={N:32768,r:8,p:3,maxmem:40*1024*1024};
let hashing=false;
export class PasswordBusy extends Error{}
export function passwordPepper(env:Env):string{const value=env.PASSWORD_PEPPER;if(!value||encoder.encode(value).length<32)throw Error('PASSWORD_CONFIG');return value}
export function passwordReady(env:Env):boolean{return env.PASSWORD_AUTH_ENABLED==='true'&&!!env.PASSWORD_PEPPER&&encoder.encode(env.PASSWORD_PEPPER).length>=32&&!!env.SESSION_HMAC_SECRET&&encoder.encode(env.SESSION_HMAC_SECRET).length>=32}
export async function passwordEmailKey(env:Env,email:string):Promise<string>{const key=await crypto.subtle.importKey('raw',encoder.encode(passwordPepper(env)),{name:'HMAC',hash:'SHA-256'},false,['sign']);return Array.from(new Uint8Array(await crypto.subtle.sign('HMAC',key,encoder.encode('kortex-password-email-v1\n'+email))),b=>b.toString(16).padStart(2,'0')).join('')}
export function validNewPassword(value:string):boolean{
 const length=[...value].length;
 return length>=15&&length<=128&&encoder.encode(value).length<=512&&!!value.trim()&&!/^([\s\S])\1+$/.test(value)&&!['passwordpassword','123456789012345','azertyazertyazerty','motdepasse123456','thisisapassword'].includes(value.toLowerCase());
}
async function derive(env:Env,password:string,salt:Uint8Array):Promise<Uint8Array>{
 if(hashing)throw new PasswordBusy();hashing=true;let material:Uint8Array|undefined;
 try{const key=await crypto.subtle.importKey('raw',encoder.encode(passwordPepper(env)),{name:'HMAC',hash:'SHA-256'},false,['sign']);material=new Uint8Array(await crypto.subtle.sign('HMAC',key,encoder.encode('kortex-password-v1\n'+password)));return await new Promise<Uint8Array>((resolve,reject)=>scrypt(material!,salt,32,COST,(error,result)=>error?reject(error):resolve(new Uint8Array(result))))}finally{material?.fill(0);hashing=false}
}
export async function hashPassword(env:Env,password:string):Promise<string>{
 const salt=crypto.getRandomValues(new Uint8Array(16)),derived=await derive(env,password,salt);
 try{return 'scrypt$v1$32768$8$3$'+b64(salt)+'$'+b64(derived)}finally{derived.fill(0)}
}
export async function verifyPassword(env:Env,password:string,stored:string|null):Promise<boolean>{
 let salt:Uint8Array=new Uint8Array(16),expected:Uint8Array=new Uint8Array(32),valid=false;
 if(stored){try{const parts=stored.split('$');if(parts.length===7&&parts.slice(0,5).join('$')==='scrypt$v1$32768$8$3'){const a=base64url(parts[5]),b=base64url(parts[6]);if(a.length===16&&b.length===32){salt=a;expected=b;valid=true}}}catch{/* A malformed hash fails closed at the normal work factor. */}}
 const actual=await derive(env,password,salt);try{return equal(b64(actual),b64(expected))&&valid}finally{actual.fill(0);expected.fill(0)}
}
