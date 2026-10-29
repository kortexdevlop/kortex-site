import type {ServiceBinding} from './service-binding.ts';
const route=/^\/api\/v1\/(?:health|trust\/revocations|runtime\/channels|tools(?:\/[a-z0-9]+(?:-[a-z0-9]+)*(?:\/actions)?)?|downloads\/[a-z0-9]+(?:-[a-z0-9]+)*\/(?:[0-9A-Za-z.+-]|%2[bB])+)$/;
function unavailable(status=503):Response {return Response.json({error:{code:status===403?'FORBIDDEN':status===405?'BAD_REQUEST':status===404?'NOT_FOUND':'UNAVAILABLE',message:status===503?'Service unavailable':'Request rejected',request_id:crypto.randomUUID()}},{status,headers:{'Cache-Control':'no-store','X-Content-Type-Options':'nosniff',...(status===405?{Allow:'GET, HEAD'}:{})}});}
export async function proxy(request:Request,binding:ServiceBinding|undefined,webOrigin:string|undefined,authBinding?:ServiceBinding):Promise<Response> {
  const url=new URL(request.url);
  if(/^\/api\/v1\/(?:auth\/(?:login|callback|logout|options|desktop\/(?:request|approve)|password\/(?:token|register|login))|ai\/plan|me(?:\/(?:library|preferences|export|data|devices|workflows(?:\/[a-zA-Z0-9_-]{1,64})?))?)$/.test(url.pathname))return authProxy(request,authBinding,webOrigin);
  if(!route.test(url.pathname))return unavailable(404);
  if(!['GET','HEAD'].includes(request.method))return unavailable(405);
  const origin=request.headers.get('Origin');
  if(!webOrigin||url.origin!==webOrigin||(origin&&origin!==webOrigin)||request.headers.get('Sec-Fetch-Site')==='cross-site')return unavailable(403);
  if(!binding)return unavailable();
  const headers=new Headers({Accept:'application/json'});
  for(const name of ['cookie','if-none-match','range','if-range']){const value=request.headers.get(name);if(value)headers.set(name,value);}
  const controller=new AbortController();let timer:ReturnType<typeof setTimeout>;
  const expired=new Promise<never>((_,reject)=>{timer=setTimeout(()=>{controller.abort();reject(new Error('API timeout'));},4000);});
  try {
    const upstream=await Promise.race([binding.fetch(new Request(`https://kortex-api.internal${url.pathname}${url.search}`,{method:request.method,headers,signal:controller.signal,redirect:'manual'})),expired]);
    if(upstream.status>=300&&upstream.status<400&&upstream.status!==304)return unavailable();
    const safe=new Headers({'Cache-Control':'private, no-store','Vary':'Cookie','X-Content-Type-Options':'nosniff','Content-Security-Policy':"default-src 'none'; frame-ancestors 'none'"});
    for(const name of ['content-type','content-disposition','content-length','content-range','accept-ranges','etag','x-request-id','x-kortex-sha256','retry-after']){const value=upstream.headers.get(name);if(value)safe.set(name,value);}
    for(const cookie of upstream.headers.getSetCookie()) {
      if(!/;\s*HttpOnly(?:;|$)/i.test(cookie)||(url.protocol==='https:'&&!/;\s*Secure(?:;|$)/i.test(cookie))||!/;\s*SameSite=(?:Lax|Strict)(?:;|$)/i.test(cookie))return unavailable();
      safe.append('Set-Cookie',cookie);
    }
    return new Response(request.method==='HEAD'||[204,304].includes(upstream.status)?null:upstream.body,{status:upstream.status,headers:safe});
  }catch{return unavailable();}finally{clearTimeout(timer!);}
}

async function authProxy(request:Request,binding:ServiceBinding|undefined,webOrigin:string|undefined):Promise<Response>{
 const url=new URL(request.url),callback=url.pathname==='/api/v1/auth/callback',login=url.pathname==='/api/v1/auth/login',logout=url.pathname==='/api/v1/auth/logout',passwordPost=/\/password\/(?:register|login)$/.test(url.pathname),extraPost=/\/(?:desktop\/approve|ai\/plan)$/.test(url.pathname),account=/\/me\/(library|preferences|export|data|devices|workflows)(?:\/[a-zA-Z0-9_-]{1,64})?$/.exec(url.pathname);
 if(!webOrigin||url.origin!==webOrigin||request.headers.get('Origin')&&request.headers.get('Origin')!==webOrigin||!callback&&request.headers.get('Sec-Fetch-Site')==='cross-site')return unavailable(403);
 const allowed=account?account[1]==='workflows'?['GET','POST','PUT','DELETE']:account[1]==='library'?['GET','PUT','DELETE']:account[1]==='preferences'?['GET','PATCH']:account[1]==='export'?['GET']:account[1]==='devices'?['GET','DELETE','PATCH']:['DELETE']:logout||passwordPost||extraPost?['POST']:['GET'];
 if(!allowed.includes(request.method)){const rejected=unavailable(405);rejected.headers.set('Allow',allowed.join(', '));return rejected;}
 if(!binding)return unavailable();
 // The Web facade accepts cookies only. Desktop bearer tokens use the API origin directly.
 if(request.headers.has('Authorization'))return unavailable(403);
 const headers=new Headers({Accept:'application/json'});
 for(const name of ['cookie','origin','sec-fetch-site','x-csrf-token','content-type','if-match','x-kortex-account']){const value=request.headers.get(name);if(value)headers.set(name,value);}
 let timer:ReturnType<typeof setTimeout>|undefined;const controller=new AbortController();
 try{
  const expired=new Promise<never>((_,reject)=>{timer=setTimeout(()=>{controller.abort();reject(Error('Auth timeout'))},35000)});
  let body:Uint8Array<ArrayBuffer>|undefined;
  if(passwordPost||extraPost||account&&['POST','PUT','PATCH'].includes(request.method)||account&&['data','devices'].includes(account[1])&&request.method==='DELETE'){
   const reader=request.body?.getReader();if(!reader)return unavailable(403);const chunks:Uint8Array[]=[];let size=0;
   try{for(;;){const part=await Promise.race([reader.read(),expired]);if(part.done)break;size+=part.value.length;if(size>(passwordPost?4096:account?.[1]==='workflows'?262144:16384))return unavailable(403);chunks.push(part.value)}}finally{void reader.cancel().catch(()=>{})}
   body=new Uint8Array(size);let at=0;for(const chunk of chunks){body.set(chunk,at);at+=chunk.length}
  }
  const upstream=await Promise.race([binding.fetch(new Request('https://kortex-api.internal'+url.pathname+url.search,{method:request.method,headers,...(body?{body}:{}),redirect:'manual',signal:controller.signal})),expired]);
  const safe=new Headers({'Cache-Control':account?'private, no-store, no-transform':'private, no-store','Vary':'Cookie','Content-Security-Policy':"default-src 'none'; frame-ancestors 'none'",'X-Content-Type-Options':'nosniff','Referrer-Policy':'no-referrer'});
  for(const name of ['content-type','x-request-id','retry-after','allow','etag','content-disposition','x-kortex-limit-source']){const value=upstream.headers.get(name);if(value)safe.set(name,value);}
  if(upstream.status>=300&&upstream.status<400){
   const target=new URL(upstream.headers.get('Location')??'');const local=new URL(webOrigin).protocol==='http:';
   const provider=target.origin==='https://accounts.google.com'||local&&target.protocol==='http:'&&['127.0.0.1','localhost','[::1]'].includes(target.hostname);
   if(!(login&&upstream.status===302&&provider)&&!(callback&&upstream.status===303&&(target.href===webOrigin+'/app'||target.origin===webOrigin&&/^\/account\/connect-desktop\?request_id=[0-9a-f-]{36}$/.test(target.pathname+target.search))))return unavailable();
   safe.set('Location',target.href);
  }
  for(const value of upstream.headers.getSetCookie()){
   if(!/^__Host-kortex_(?:session|oidc|password)=[A-Za-z0-9_-]*;/.test(value)||/;\s*Domain=/i.test(value)||!/;\s*Path=\/(?:;|$)/i.test(value)||!/;\s*Secure(?:;|$)/i.test(value)||!/;\s*HttpOnly(?:;|$)/i.test(value)||!/;\s*SameSite=Lax(?:;|$)/i.test(value))return unavailable();
   safe.append('Set-Cookie',value);
  }
  return new Response(upstream.status===204?null:upstream.body,{status:upstream.status,headers:safe});
 }catch{return unavailable()}finally{if(timer!==undefined)clearTimeout(timer)}
}
