import {defineMiddleware} from 'astro:middleware';
import {env} from 'cloudflare:workers';
import {accountView} from './lib/auth';
import {accountReturn} from './lib/return-to';
import {assistantCsp} from './lib/ai-csp';
export const onRequest=defineMiddleware(async(context,next)=>{
 let response:Response;const privatePage=/^\/account(?:\/|$)/.test(context.url.pathname);
 if(privatePage){try{const user=await accountView(context.request,env.AUTH_WORKER,env.WEB_ORIGIN);if(!user){const next=accountReturn(context.url.pathname+context.url.search);response=context.redirect('/auth/login'+(next!=='/account'?'?returnTo='+encodeURIComponent(next):''),302)}else{context.locals.account=user;response=await next()}}catch(error){console.error('ACCOUNT_SSR_FAILURE',error instanceof Error?error.name:'unavailable');response=new Response('Votre compte est momentanément indisponible.',{status:503,headers:{'Content-Type':'text/plain; charset=utf-8'}})}}else response=await next();
 response.headers.set('Strict-Transport-Security','max-age=31536000');response.headers.set('X-Frame-Options','DENY');response.headers.set('X-Content-Type-Options','nosniff');
 if(privatePage){response.headers.set('Cache-Control','private, no-store');response.headers.set('Vary','Cookie');response.headers.set('Referrer-Policy','no-referrer');response.headers.set('X-Robots-Tag','noindex, nofollow')}
 if(env.WEB_ORIGIN==='https://kortex-web-staging.userpgl.workers.dev')response.headers.set('X-Robots-Tag','noindex, nofollow');if(['/ai','/ai/','/workflows','/workflows/','/workspace','/workspace/'].includes(context.url.pathname)&&response.headers.get('Content-Type')?.includes('text/html'))return assistantCsp(response);return response;
});
