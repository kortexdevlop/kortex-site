import type {Env} from '../env';
// Documents signed ahead of time by the offline roots. The API holds no private key:
// it only picks the document valid right now, and still verifies it before serving it.
export const TRUST_SCHEDULE_KEY='trust/schedule.json';
export interface TrustSchedule {revocations:unknown[];channels:unknown[]}
const LIMIT=1048576,MAX_AGE=600000;
let cache:{at:number;value:TrustSchedule}|null=null;
export function clearTrustSchedule(){cache=null}
export async function trustSchedule(env:Env):Promise<TrustSchedule>{
 const now=Date.now();if(cache&&now-cache.at<MAX_AGE)return cache.value;
 let value:TrustSchedule={revocations:[],channels:[]};
 try{const object=await env.TOOLS_BUCKET.get(TRUST_SCHEDULE_KEY);if(object&&object.size<=LIMIT){const parsed=JSON.parse(await object.text()) as Partial<TrustSchedule>;value={revocations:Array.isArray(parsed.revocations)?parsed.revocations:[],channels:Array.isArray(parsed.channels)?parsed.channels:[]}}}catch{}
 cache={at:now,value};return value;
}
