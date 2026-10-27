import type {APIRoute} from 'astro';
import {env} from 'cloudflare:workers';
export const prerender=false;
export const GET:APIRoute=()=>new Response(`User-agent: *\nAllow: /\nDisallow: /app/\nDisallow: /api/\nDisallow: /account/\nDisallow: /__dev/\nSitemap: ${env.WEB_ORIGIN}/sitemap-index.xml\n`,{headers:{'Content-Type':'text/plain; charset=utf-8','Cache-Control':'public, max-age=900'}});
