const hex=(bytes:ArrayBuffer)=>Array.from(new Uint8Array(bytes),b=>b.toString(16).padStart(2,'0')).join('');
export async function sessionHash(token:string,secret:string):Promise<string>{
 if(!/^[A-Za-z0-9_-]{43}$/.test(token)||new TextEncoder().encode(secret).length<32)throw Error('SESSION_INPUT');
 const key=await crypto.subtle.importKey('raw',new TextEncoder().encode(secret),{name:'HMAC',hash:'SHA-256'},false,['sign']);
 return hex(await crypto.subtle.sign('HMAC',key,new TextEncoder().encode('kortex-session-v1\n'+token)));
}
export async function activeSession(db:D1Database,hash:string,now:number){
 if(!/^[a-f0-9]{64}$/.test(hash)||!Number.isSafeInteger(now)||now<0)throw Error('SESSION_INPUT');
 return db.prepare('SELECT user_id,expires_at FROM sessions WHERE token_hash=? AND expires_at>? AND revoked_at IS NULL').bind(hash,now).first<{user_id:string;expires_at:number}>();
}
export async function expireIdentityData(db:D1Database,now:number){
 if(!Number.isSafeInteger(now)||now<0)throw Error('SESSION_INPUT');
 return db.batch([db.prepare('DELETE FROM sessions WHERE expires_at<=? OR revoked_at IS NOT NULL').bind(now),db.prepare('DELETE FROM auth_events WHERE expires_at<=?').bind(now)]);
}
