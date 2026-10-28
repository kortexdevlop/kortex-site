import {defineMiddleware} from 'astro:middleware';
import {env} from 'cloudflare:workers';
import {accountView} from './lib/auth';
export const onRequest=defineMiddleware(async(context,next)=>{
 let response:Response;const privatePage=/^\/account(?:\/|$)/.test(context.url.pathname);
 if(privatePage){try{const user=await accountView(context.request,env.AUTH_WORKER,env.WEB_ORIGIN);if(!user)response=context.redirect('/auth/login',302);else{context.locals.account=user;response=await next()}}catch(error){console.error('ACCOUNT_SSR_FAILURE',error instanceof Error?error.name+': '+error.message:'unavailable');response=new Response('Votre compte est momentanément indisponible.',{status:503,headers:{'Content-Type':'text/plain; charset=utf-8'}})}}else response=await next();
 if(privatePage){response.headers.set('Cache-Control','private, no-store');response.headers.set('Vary','Cookie');response.headers.set('Referrer-Policy','no-referrer');response.headers.set('X-Robots-Tag','noindex, nofollow')}
 if(env.WEB_ORIGIN==='https://kortex-web-staging.userpgl.workers.dev')response.headers.set('X-Robots-Tag','noindex, nofollow');return response;
});
