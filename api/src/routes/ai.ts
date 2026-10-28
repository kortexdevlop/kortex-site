import type {Env} from '../env';
import {failure,json} from '../lib/http';
import {readBounded,AdminError} from '../lib/admin-error';
import {userScope} from '../security/user-scope';
import {reserveAi} from '../security/ai-rate-limit';
import {aiConfig,scrubMessage} from '../services/ai-config';
import {aiRequest,makeProposal} from '../services/ai';
export async function ai(request:Request,env:Env,id:string,viaWeb:boolean):Promise<Response|null>{
 const url=new URL(request.url);if(url.pathname!=='/api/v1/ai/plan')return null;if(request.method!=='POST')return failure('BAD_REQUEST','Method not allowed',405,id);
 try{
  const owner=await userScope(request,env,viaWeb,true,'ai:plan'),config=aiConfig(env);if(!config)return failure('UNAVAILABLE','AI proposals are disabled',503,id);
  if(url.searchParams.size||request.headers.get('Content-Type')?.split(';')[0]!=='application/json')return failure('BAD_REQUEST','Invalid AI request',400,id);
  const parsed=aiRequest.safeParse(JSON.parse(new TextDecoder().decode(await readBounded(request.body,8192))));if(!parsed.success)return failure('BAD_REQUEST','Only consented messages without private files are supported',400,id);
  try{scrubMessage(parsed.data.message)}catch{return failure('BAD_REQUEST','Remove private data or secrets from the message',400,id)}
  if(!await reserveAi(env,owner.user_id,config))return failure('RATE_LIMITED','Kortex application daily limit reached; this does not indicate Cloudflare quota usage',429,id,undefined,false,{'Retry-After':String(Math.ceil((86400000-Date.now()%86400000)/1000)),'X-Kortex-Limit-Source':'application'});
  try{return json(await makeProposal(env,parsed.data),200,id)}catch(error){
   // Only fixed categories / numeric provider codes escape; never prompts, model output or exception text.
   const message=error instanceof Error?error.message:'';
   const diagnostic=/^AI_[A-Z_]+$/.test(message)?message:error instanceof Error&&error.name==='ZodError'?'AI_INVALID_PLAN':'AI_PROVIDER_REJECTED';
   const providerCode=diagnostic==='AI_PROVIDER_REJECTED'?/\b(\d{4,6})\b/.exec(message)?.[1]:undefined;
   return failure('UNAVAILABLE',diagnostic+(providerCode?'_'+providerCode:'')+'; nothing was executed',503,id);
  }
 }catch(error){if(error instanceof AdminError)return failure(error.code,error.message,error.status,id);return failure('BAD_REQUEST','Invalid AI request',400,id)}
}
