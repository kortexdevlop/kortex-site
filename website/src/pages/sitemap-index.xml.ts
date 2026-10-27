import type {APIRoute} from 'astro';
import {env} from 'cloudflare:workers';
import {sitemapPage,xml,xmlResponse} from '../lib/seo';
export const prerender=false;
export const GET:APIRoute=async()=>{try{const value=await sitemapPage(env.SITEMAP_WORKER,0),origin=env.WEB_ORIGIN!;return xmlResponse('<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'+Array.from({length:value.pages},(_,p)=>`<sitemap><loc>${xml(origin+'/sitemaps/'+p+'.xml')}</loc></sitemap>`).join('')+'</sitemapindex>');}catch{return new Response('Service indisponible',{status:503,headers:{'Cache-Control':'no-store'}});}};
