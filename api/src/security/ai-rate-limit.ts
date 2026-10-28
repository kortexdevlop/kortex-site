import type {Env} from '../env';
import {opaqueHash} from './sessions';
import type {aiConfig} from '../services/ai-config';
export async function reserveAi(env:Env,user:string,config:NonNullable<ReturnType<typeof aiConfig>>){
 const now=Date.now(),day=Math.floor(now/86400000),expiry=(day+1)*86400000,userKey=await opaqueHash(env,'kortex-ai-quota-v1',user+':'+day),global='ai-global-'+day,nonce=crypto.randomUUID();
 await env.DB.prepare('DELETE FROM ai_quota WHERE expires_at<=?').bind(now).run();
 const rows=await env.DB.batch([
 env.DB.prepare(`INSERT INTO ai_quota(bucket,requests,reserved_tokens,expires_at,last_reservation) SELECT ?,1,?,?,? WHERE coalesce((SELECT requests FROM ai_quota WHERE bucket=?),0)<? ON CONFLICT(bucket) DO UPDATE SET requests=requests+1,reserved_tokens=reserved_tokens+excluded.reserved_tokens,last_reservation=excluded.last_reservation WHERE requests<? AND reserved_tokens+excluded.reserved_tokens<=?`).bind(global,config.reservation,expiry,nonce,userKey,config.user,config.global,config.tokens),
 env.DB.prepare(`INSERT INTO ai_quota(bucket,requests,reserved_tokens,expires_at,last_reservation) SELECT ?,1,?,?,? WHERE EXISTS(SELECT 1 FROM ai_quota WHERE bucket=? AND last_reservation=?) ON CONFLICT(bucket) DO UPDATE SET requests=requests+1,reserved_tokens=reserved_tokens+excluded.reserved_tokens,last_reservation=excluded.last_reservation`).bind(userKey,config.reservation,expiry,nonce,global,nonce)]);
 return rows[0].meta.changes===1;
}
