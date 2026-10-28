import type {APIRoute} from 'astro';
import {env} from 'cloudflare:workers';
import {proxy} from '../../lib/proxy';
export const prerender=false;
export const GET:APIRoute=({request})=>{
 const url=new URL(request.url);url.pathname='/api/v1/auth/callback';
 return proxy(new Request(url,{method:'GET',headers:request.headers,redirect:'manual'}),env.API_WORKER,env.WEB_ORIGIN,env.AUTH_WORKER);
};
