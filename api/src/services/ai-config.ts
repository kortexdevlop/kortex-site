import {z} from 'zod';
import type {Env} from '../env';
export const MODEL='@cf/meta/llama-3.3-70b-instruct-fp8-fast';
export function aiConfig(env:Env){
 if(env.AI_ENABLED!=='true'||!env.AI||env.AI_MODEL!==MODEL)return null;
 const config=z.object({user:z.coerce.number().int().min(1).max(10),global:z.coerce.number().int().min(1).max(20),tokens:z.coerce.number().int().min(16896).max(337920)}).parse({user:env.AI_USER_DAILY_REQUESTS??'2',global:env.AI_GLOBAL_DAILY_REQUESTS??'3',tokens:env.AI_DAILY_RESERVED_TOKENS??'50688'});
 return {...config,model:MODEL,output:512,reservation:16896};
}
export function scrubMessage(message:string):string{
 if(/-----BEGIN|\bBearer\s|\b(?:password|api[_ -]?key|access[_ -]?token|mot de passe)\s*[:=]|(?:[A-Z]:\\|\/Users\/|\/home\/)|data:[^;]+;base64|https?:\/\/[^\s/]+:[^\s/]+@/i.test(message))throw Error('PRIVATE_DATA');
 return message.replace(/[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}/gi,'[EMAIL]').replace(/\+\d[\d .()-]{7,}\d/g,'[TELEPHONE]').replace(/\b0[1-9](?:[ .-]?\d{2}){4}\b/g,'[TELEPHONE]');
}
