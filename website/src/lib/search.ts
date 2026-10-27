import {ApiClient} from './api-client';
/** Latest response wins; stale requests are cancelled even when transport ignores abort. */
export function liveSearch(input:HTMLInputElement,show:(state:'loading'|'ready'|'error',items?:{id:string;name:string}[])=>void){
 let timer:ReturnType<typeof setTimeout>,controller:AbortController|undefined,generation=0;
 const change=()=>{clearTimeout(timer);controller?.abort();const current=++generation,q=input.value.trim().slice(0,100);if(!q){show('ready',[]);return;}
 timer=setTimeout(async()=>{controller=new AbortController();const signal=controller.signal;show('loading');try{const client=new ApiClient(undefined,(url,init)=>fetch(url,{...init,signal:AbortSignal.any([signal,init?.signal||signal])}));const result=await client.listTools(new URLSearchParams({q,limit:'8'}));if(current===generation&&!signal.aborted)show('ready',result.items);}catch{if(current===generation&&!signal.aborted)show('error');}},250);};
 input.addEventListener('input',change);return ()=>{generation++;clearTimeout(timer);controller?.abort();input.removeEventListener('input',change);};
}
