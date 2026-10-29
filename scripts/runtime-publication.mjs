import {verifyRuntime} from '../shared/typescript/src/runtime.ts';

export async function waitRuntimePublication(url,root,expected,{fetcher=fetch,delay=ms=>new Promise(r=>setTimeout(r,ms)),attempts=30}={}){
 for(let attempt=0;attempt<attempts;attempt++){
  const response=await fetcher(url,{cache:'no-store',redirect:'error',signal:AbortSignal.timeout(15000)});
  if(response.ok){
   const checked=await verifyRuntime(await response.json(),root);
   if(checked.sequence===expected.sequence&&checked.signature===expected.signature)return checked;
   if(checked.sequence>=expected.sequence)throw Error('RUNTIME_CHANNEL_MISMATCH');
  }else if(response.status!==503)throw Error('RUNTIME_CHANNEL_NOT_VISIBLE');
  if(attempt+1<attempts)await delay(1000);
 }
 throw Error('RUNTIME_CHANNEL_PROPAGATION_TIMEOUT');
}
