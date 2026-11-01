import type {Env} from '../env';
import {opaqueHash} from './sessions';
import {WINDOW_MS,type aiConfig} from '../services/ai-config';
type Config=NonNullable<ReturnType<typeof aiConfig>>;
export interface Reservation {user:string;global:string;neurons:number}
const windowOf=(now:number)=>Math.floor(now/WINDOW_MS),dayOf=(now:number)=>Math.floor(now/86400000);
export const accountBucket=(env:Env,user:string,now=Date.now())=>opaqueHash(env,'kortex-ai-neurons-v1',user+':'+windowOf(now));
const upsert=`INSERT INTO ai_neurons(bucket,used,expires_at,reservation) SELECT ?,?,?,? WHERE `;
/** Holds the worst case before inference; settleAi gives back what the provider did not use. */
export async function reserveAi(env:Env,user:string,config:Config,neurons:number):Promise<Reservation|'account'|'service'>{
 const now=Date.now(),userKey=await accountBucket(env,user,now),global='ai-neurons-'+dayOf(now),nonce=crypto.randomUUID();
 await env.DB.prepare('DELETE FROM ai_neurons WHERE expires_at<=?').bind(now).run();
 const rows=await env.DB.batch([
 env.DB.prepare(upsert+`coalesce((SELECT used FROM ai_neurons WHERE bucket=?),0)+?<=? AND coalesce((SELECT used FROM ai_neurons WHERE bucket=?),0)+?<=? ON CONFLICT(bucket) DO UPDATE SET used=used+excluded.used,reservation=excluded.reservation`).bind(global,neurons,(dayOf(now)+1)*86400000,nonce,userKey,neurons,config.user,global,neurons,config.global),
 env.DB.prepare(upsert+`EXISTS(SELECT 1 FROM ai_neurons WHERE bucket=? AND reservation=?) ON CONFLICT(bucket) DO UPDATE SET used=used+excluded.used,reservation=excluded.reservation`).bind(userKey,neurons,(windowOf(now)+1)*WINDOW_MS,nonce,global,nonce)]);
 if(rows[0].meta.changes===1)return {user:userKey,global,neurons};
 const row=await env.DB.prepare('SELECT used FROM ai_neurons WHERE bucket=?').bind(userKey).first<{used:number}>();
 return (row?.used??0)+neurons>config.user?'account':'service';
}
export async function settleAi(env:Env,reservation:Reservation,actual:number){
 const delta=actual-reservation.neurons;if(delta)await env.DB.prepare('UPDATE ai_neurons SET used=max(0,used+?) WHERE bucket IN (?,?)').bind(delta,reservation.user,reservation.global).run();
}
export async function aiQuota(env:Env,user:string,config:Config){
 const now=Date.now(),rows=await env.DB.prepare('SELECT bucket,used FROM ai_neurons WHERE bucket IN (?,?) AND expires_at>?').bind(await accountBucket(env,user,now),'ai-neurons-'+dayOf(now),now).all<{bucket:string;used:number}>();
 const used=rows.results.find(r=>!r.bucket.startsWith('ai-neurons-'))?.used??0,global=rows.results.find(r=>r.bucket.startsWith('ai-neurons-'))?.used??0;
 return {limit:config.user,used:Math.min(used,config.user),remaining:Math.max(0,config.user-used),resets_at:(windowOf(now)+1)*WINDOW_MS,window_hours:3,service_available:global<config.global};
}
export const retryAt=(scope:'account'|'service',now=Date.now())=>scope==='account'?(windowOf(now)+1)*WINDOW_MS:(dayOf(now)+1)*86400000;
