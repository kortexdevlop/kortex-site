import type {APIRoute} from 'astro';
import {env} from 'cloudflare:workers';
import {sitemapPage,xml,xmlResponse} from '../../lib/seo';
export const prerender=false;
export const GET:APIRoute=async({params})=>{if(!/^(0|[1-9][0-9]{0,4})$/.test(params.page||''))return new Response(null,{status:404});try{const p=Number(params.page),value=await sitemapPage(env.SITEMAP_WORKER,p);if(p>=value.pages)return new Response(null,{status:404,headers:{'Cache-Control':'no-store'}});return xmlResponse('<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">'+value.items.map(t=>`<url><loc>${xml(env.WEB_ORIGIN+'/tools/'+encodeURIComponent(t.id))}</loc><lastmod>${new Date(t.published_at).toISOString()}</lastmod></url>`).join('')+'</urlset>');}catch{return new Response('Service indisponible',{status:503,headers:{'Cache-Control':'no-store'}});}};
