import {z} from 'zod';
import type {Env} from '../env';
export const MODEL='@cf/qwen/qwen3-30b-a3b-fp8';
// Published Workers AI rates, in neurons per million tokens. Update them with MODEL.
export const RATES={input:4625,output:30475};
export const WINDOW_MS=10800000;
export function aiConfig(env:Env){
 if(env.AI_ENABLED!=='true'||!env.AI||env.AI_MODEL!==MODEL)return null;
 const config=z.object({user:z.coerce.number().int().min(50).max(2000),global:z.coerce.number().int().min(100).max(10000)}).parse({user:env.AI_USER_WINDOW_NEURONS??'200',global:env.AI_GLOBAL_DAILY_NEURONS??'9000'});
 return {...config,model:MODEL,output:500};
}
export function neuronsFor(input:number,output:number){return Math.ceil((input*RATES.input+output*RATES.output)/1000000)}
export function scrubMessage(message:string):string{
 if(/-----BEGIN|\bBearer\s|\b(?:password|api[_ -]?key|access[_ -]?token|mot de passe)\s*[:=]|(?:[A-Z]:\\|\/Users\/|\/home\/)|data:[^;]+;base64|https?:\/\/[^\s/]+:[^\s/]+@/i.test(message))throw Error('PRIVATE_DATA');
 return message.replace(/[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi,'[EMAIL]').replace(/\+\d[\d .()-]{7,}\d/g,'[TELEPHONE]').replace(/\b0[1-9](?:[ .-]?\d{2}){4}\b/g,'[TELEPHONE]');
}
