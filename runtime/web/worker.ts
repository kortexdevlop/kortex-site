import {ApiClient,type ServiceBinding} from '../../website/src/lib/api-client';
import {frameHeaders} from '../../web-adapter/src/iframe';
interface Env {ASSETS:ServiceBinding;API_WORKER:ServiceBinding;PUBLIC_WEB_ORIGIN:string;RUN_ORIGIN:string;TRUST_KEYS:string;REVOCATION_ROOT:string;}
export default {async fetch(request:Request,env:Env):Promise<Response>{
 const url=new URL(request.url),headers={...frameHeaders(env.PUBLIC_WEB_ORIGIN),'X-Robots-Tag':'noindex, nofollow','Strict-Transport-Security':'max-age=31536000'};
 if(!['GET','HEAD'].includes(request.method))return new Response(null,{status:405,headers});
 if(url.pathname==='/frame.html'||url.pathname==='/embed-config.js'){
  const id=url.searchParams.get('tool')||'',version=url.searchParams.get('version')||'';
  if(!/^[a-z][a-z0-9-]{2,63}$/.test(id)||!/^\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.+-]+)?$/.test(version))return new Response(null,{status:400,headers});
  try{const detail=await new ApiClient(env.API_WORKER).getTool(id);if(detail.tool.version!==version)return new Response(null,{status:404,headers});
   if(url.pathname==='/embed-config.js'){
    const config={parent_origin:env.PUBLIC_WEB_ORIGIN,api_origin:env.RUN_ORIGIN,keys:JSON.parse(env.TRUST_KEYS),revocation_root:JSON.parse(env.REVOCATION_ROOT),artifacts:[{...detail.descriptor,url:env.RUN_ORIGIN+'/api/v1/downloads/'+id+'/'+encodeURIComponent(version)}]};
    return new Response('window.kortexEmbedConfig='+JSON.stringify(config).replace(/</g,'\\u003c')+';',{headers:{...headers,'Content-Type':'text/javascript; charset=utf-8','Cache-Control':'no-store'}});
   }
   const build=await(await env.ASSETS.fetch(new Request('https://assets.internal/version.json'))).json() as {runtime_version:string;script_integrity:string};if(url.searchParams.has('runtime')&&url.searchParams.get('runtime')!==build.runtime_version)return new Response('Runtime update required',{status:409,headers});if(!/^sha256-[A-Za-z0-9+/]{43}=$/.test(build.script_integrity))throw Error('Invalid runtime integrity');
   const script='/embed-config.js?tool='+encodeURIComponent(id)+'&amp;version='+encodeURIComponent(version);
   return new Response(`<!doctype html><html lang="fr"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Kortex — Outil isolé</title><script src="${script}"></script><link rel="stylesheet" href="/runtime-style.css"><link rel="stylesheet" href="/tool-shell.css"></head><body><div id="main"></div><script type="module" src="/wasm/kortex-runtime-web.js" integrity="${build.script_integrity}" crossorigin="anonymous"></script></body></html>`,{headers:{...headers,'Content-Type':'text/html; charset=utf-8','Cache-Control':'no-store'}});
  }catch{return new Response('Outil indisponible',{status:404,headers:{...headers,'Cache-Control':'no-store'}});}
 }
 if(url.pathname==='/api/v1/trust/revocations'||/^\/api\/v1\/downloads\/[a-z][a-z0-9-]{2,63}\/[^/]+$/.test(url.pathname)){
  const upstream=await env.API_WORKER.fetch(new Request('https://kortex-api.internal'+url.pathname,{redirect:'manual'}));
  const safe=new Headers(headers);for(const name of ['content-type','content-length','x-kortex-sha256']){const value=upstream.headers.get(name);if(value)safe.set(name,value);}safe.set('Cache-Control','no-store');
  return new Response(request.method==='HEAD'?null:upstream.body,{status:upstream.status,headers:safe});
 }
 if(url.pathname==='/'||url.pathname.includes('..'))return new Response(null,{status:404,headers});
 const asset=await env.ASSETS.fetch(request),safe=new Headers(asset.headers);for(const [key,value] of Object.entries(headers))safe.set(key,value);
 if(url.pathname==='/version.json')safe.set('Cache-Control','no-store');return new Response(asset.body,{status:asset.status,headers:safe});
}};
