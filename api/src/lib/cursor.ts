import {z} from 'zod';
export class BadQuery extends Error {}
export class CatalogUnavailable extends Error {}
export const slug=z.string().min(3).max(64).regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/);
const scope=z.object({sort:z.enum(['recent','name']),limit:z.number().int().min(1).max(100),q:z.string().max(200),category:z.string().max(64),tag:z.string().max(40)}).strict();
export type Scope=z.infer<typeof scope>;
const payload=z.object({v:z.literal(1),scope,after:z.object({id:slug,key:z.union([z.string().min(1).max(200),z.number().int().nonnegative().max(Number.MAX_SAFE_INTEGER)])}).strict()}).strict();
export type Position=z.infer<typeof payload>['after'];
const encoder=new TextEncoder();
function encode(bytes:Uint8Array):string {return btoa(String.fromCharCode(...bytes)).replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'')}
function decode(text:string):Uint8Array {
 if(!/^[A-Za-z0-9_-]+$/.test(text))throw new BadQuery();
 const bytes=Uint8Array.from(atob(text.replaceAll('-','+').replaceAll('_','/')),c=>c.charCodeAt(0));
 if(encode(bytes)!==text)throw new BadQuery();return bytes;
}
export async function cursorKey(secret:unknown):Promise<CryptoKey> {
 // A dedicated, random, 256-bit server secret. No public or artifact signing key reuse.
 if(typeof secret!=='string'||!/^[0-9a-f]{64}$/.test(secret))throw new CatalogUnavailable();
 const bytes=Uint8Array.from(secret.match(/../g)!,s=>parseInt(s,16));
 return crypto.subtle.importKey('raw',bytes,{name:'HMAC',hash:'SHA-256'},false,['sign','verify']);
}
export async function signCursor(scope:Scope,after:Position,key:CryptoKey):Promise<string> {
 const body=encode(encoder.encode(JSON.stringify({v:1,scope,after})));
 return body+'.'+encode(new Uint8Array(await crypto.subtle.sign('HMAC',key,encoder.encode(body))));
}
export async function readCursor(text:string,scope:Scope,key:CryptoKey):Promise<Position> {
 try {
  if(text.length>4096)throw new BadQuery();const parts=text.split('.');if(parts.length!==2)throw new BadQuery();
  const signature=decode(parts[1]);if(signature.length!==32||!await crypto.subtle.verify('HMAC',key,signature,encoder.encode(parts[0])))throw new BadQuery();
  const value=payload.parse(JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(decode(parts[0]))));
  if(JSON.stringify(value.scope)!==JSON.stringify(scope)||(scope.sort==='recent'?typeof value.after.key!=='number':typeof value.after.key!=='string'))throw new BadQuery();
  return value.after;
 }catch{throw new BadQuery()}
}
