import type {APIRoute} from 'astro';
import {env} from 'cloudflare:workers';
import {proxy} from '../../lib/proxy';
export const prerender=false;
export const ALL:APIRoute=({request})=>proxy(request,env.API_WORKER,env.WEB_ORIGIN);
