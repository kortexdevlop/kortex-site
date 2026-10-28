import type {Env} from '../env';
import {sessionHash,activeSession} from '../services/identity-store';
export const SESSION_COOKIE='__Host-kortex_session',FLOW_COOKIE='__Host-kortex_oidc';
export const SESSION_TTL=604800000,FLOW_TTL=600000;
export const encoder=new TextEncoder();
export function b64(bytes:Uint8Array):string{return btoa(String.fromCharCode(...bytes)).replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'')}
export function randomToken():string{return b64(crypto.getRandomValues(new Uint8Array(32)))}
export function cookie(request:Request,name:string):string|null{
 const matches=(request.headers.get('Cookie')??'').split(';').map(s=>s.trim()).filter(s=>s.startsWith(name+'='));
 if(matches.length!==1)return null;const value=matches[0].slice(name.length+1);return /^[A-Za-z0-9_-]{43}$/.test(value)?value:null;
}
export function setCookie(name:string,value:string,maxAge:number):string{return `${name}=${value}; Path=/; Max-Age=${maxAge}; Secure; HttpOnly; SameSite=Lax`}
export function secret(env:Env):string{const value=env.SESSION_HMAC_SECRET;if(!value||encoder.encode(value).length<32)throw Error('IDENTITY_CONFIG');return value}
export async function opaqueHash(env:Env,domain:string,value:string):Promise<string>{
 const key=await crypto.subtle.importKey('raw',encoder.encode(secret(env)),{name:'HMAC',hash:'SHA-256'},false,['sign']);
 return Array.from(new Uint8Array(await crypto.subtle.sign('HMAC',key,encoder.encode(domain+'\n'+value))),b=>b.toString(16).padStart(2,'0')).join('');
}
async function encryptionKey(env:Env){return crypto.subtle.importKey('raw',await crypto.subtle.digest('SHA-256',encoder.encode('kortex-oidc-encryption-v1\n'+secret(env))),'AES-GCM',false,['encrypt','decrypt'])}
export async function encryptVerifier(env:Env,value:string):Promise<string>{const iv=crypto.getRandomValues(new Uint8Array(12));const encrypted=new Uint8Array(await crypto.subtle.encrypt({name:'AES-GCM',iv,additionalData:encoder.encode('kortex-oidc-v1')},await encryptionKey(env),encoder.encode(value)));return b64(iv)+'.'+b64(encrypted)}
export async function decryptVerifier(env:Env,value:string):Promise<string>{const [iv,data]=value.split('.');const decode=(s:string)=>Uint8Array.from(atob(s.replaceAll('-','+').replaceAll('_','/')),c=>c.charCodeAt(0));return new TextDecoder().decode(await crypto.subtle.decrypt({name:'AES-GCM',iv:decode(iv),additionalData:encoder.encode('kortex-oidc-v1')},await encryptionKey(env),decode(data)))}
export async function session(request:Request,env:Env){const token=cookie(request,SESSION_COOKIE);if(!token)return null;const hash=await sessionHash(token,secret(env)),row=await activeSession(env.DB,hash,Date.now());return row?{...row,hash,token}:null}
export async function rotateSession(request:Request,env:Env,userId:string):Promise<string>{
 const token=randomToken(),hash=await sessionHash(token,secret(env)),now=Date.now(),old=cookie(request,SESSION_COOKIE);
 const writes=[env.DB.prepare('INSERT INTO sessions(token_hash,user_id,created_at,expires_at) VALUES(?,?,?,?)').bind(hash,userId,now,now+SESSION_TTL)];
 if(old)writes.push(env.DB.prepare('UPDATE sessions SET revoked_at=? WHERE token_hash=? AND revoked_at IS NULL').bind(now,await sessionHash(old,secret(env))));
 writes.push(env.DB.prepare('DELETE FROM sessions WHERE expires_at<=?').bind(now));writes.push(env.DB.prepare('DELETE FROM sessions WHERE user_id=? AND token_hash<>? AND token_hash NOT IN(SELECT token_hash FROM sessions WHERE user_id=? AND token_hash<>? ORDER BY created_at DESC,token_hash DESC LIMIT 99)').bind(userId,hash,userId,hash));await env.DB.batch(writes);return token;
}
