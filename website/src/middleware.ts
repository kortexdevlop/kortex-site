import {defineMiddleware} from 'astro:middleware';
import {env} from 'cloudflare:workers';
export const onRequest=defineMiddleware(async(_context,next)=>{const response=await next();if(env.WEB_ORIGIN==='https://kortex-web-staging.userpgl.workers.dev')response.headers.set('X-Robots-Tag','noindex, nofollow');return response;});
