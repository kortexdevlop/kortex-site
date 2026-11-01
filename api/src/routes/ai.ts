import type {Env} from '../env';
import {failure,json} from '../lib/http';
import {readBounded,AdminError} from '../lib/admin-error';
import {userScope} from '../security/user-scope';
import {aiQuota,reserveAi,retryAt,settleAi} from '../security/ai-rate-limit';
import {aiConfig} from '../services/ai-config';
import {aiRequest,chatMessages,chatRequest,converse,legacyPlan,worstCase} from '../services/ai';
function refused(scope:'account'|'service',id:string){
 const at=retryAt(scope);
 return failure('RATE_LIMITED',scope==='account'?'Kortex account neuron limit reached for this period':'Kortex daily neuron budget reached',429,id,undefined,false,{'Retry-After':String(Math.max(1,Math.ceil((at-Date.now())/1000))),'X-Kortex-Limit-Source':'application','X-Kortex-Limit-Scope':scope});
}
function unavailable(error:unknown,id:string){
 // Only fixed categories / numeric provider codes escape; never prompts, model output or exception text.
 const message=error instanceof Error?error.message:'';
 const diagnostic=/^AI_[A-Z_]+$/.test(message)?message:error instanceof Error&&error.name==='ZodError'?'AI_INVALID_PLAN':'AI_PROVIDER_REJECTED';
 const providerCode=diagnostic==='AI_PROVIDER_REJECTED'?/\b(\d{4,6})\b/.exec(message)?.[1]:undefined;
 return failure('UNAVAILABLE',diagnostic+(providerCode?'_'+providerCode:'')+'; nothing was executed',503,id);
}
export async function ai(request:Request,env:Env,id:string,viaWeb:boolean):Promise<Response|null>{
 const url=new URL(request.url),path=url.pathname;if(path!=='/api/v1/ai/plan'&&path!=='/api/v1/ai/chat'&&path!=='/api/v1/ai/quota')return null;
 const quota=path==='/api/v1/ai/quota';if(request.method!==(quota?'GET':'POST'))return failure('BAD_REQUEST','Method not allowed',405,id);
 try{
  const owner=await userScope(request,env,viaWeb,!quota,'ai:plan'),config=aiConfig(env);if(!config)return failure('UNAVAILABLE','AI proposals are disabled',503,id);
  if(quota)return json(await aiQuota(env,owner.user_id,config),200,id);
  if(url.searchParams.size||request.headers.get('Content-Type')?.split(';')[0]!=='application/json')return failure('BAD_REQUEST','Invalid AI request',400,id);
  const body=JSON.parse(new TextDecoder().decode(await readBounded(request.body,20480)));
  const legacy=path==='/api/v1/ai/plan',plain=legacy?aiRequest.safeParse(body):null,parsed=chatRequest.safeParse(plain?.success?{messages:[{role:'user',content:plain.data.message}],available_tool_ids:plain.data.available_tool_ids,consent_context:plain.data.consent_context}:body);
  if(!parsed.success||legacy&&!plain?.success)return failure('BAD_REQUEST','Only consented messages without private files are supported',400,id);
  let prepared;
  try{prepared=await chatMessages(env,parsed.data)}catch(error){if(error instanceof Error&&error.message==='PRIVATE_DATA')return failure('BAD_REQUEST','Remove private data or secrets from the message',400,id);return unavailable(error,id)}
  const reservation=await reserveAi(env,owner.user_id,config,worstCase(env,prepared.messages));if(typeof reservation==='string')return refused(reservation,id);
  try{
   const answer=await converse(env,prepared,parsed.data.attachments.length);
   if(answer.neurons!==undefined)await settleAi(env,reservation,answer.neurons);
   if(legacy)return json(legacyPlan(answer,prepared.candidates),200,id);
   return json({v:2,reply:answer.reply,steps:answer.steps,quota:await aiQuota(env,owner.user_id,config)},200,id);
  }catch(error){return unavailable(error,id)}
 }catch(error){if(error instanceof AdminError)return failure(error.code,error.message,error.status,id);return failure('BAD_REQUEST','Invalid AI request',400,id)}
}
