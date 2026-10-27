import type {ServiceBinding} from './api-client';
import type {ToolDetail} from './tools';
import {canonical} from './urls.ts';
export const xml=(s:string)=>s.replace(/[<>&"']/g,c=>({'<':'&lt;','>':'&gt;','&':'&amp;','"':'&quot;',"'":'&apos;'}[c]!));
export function software(detail:ToolDetail,origin:string){return {'@context':'https://schema.org','@type':'SoftwareApplication',name:detail.tool.name,description:detail.manifest.seo.description,applicationCategory:detail.tool.category,operatingSystem:'Web',softwareVersion:detail.tool.version,url:canonical('/tools/'+encodeURIComponent(detail.tool.id),origin)};}
export function jsonLd(value:unknown){return JSON.stringify(value).replace(/</g,'\\u003c');}
export async function sitemapPage(binding:ServiceBinding|undefined,page:number){
 if(!binding)throw Error('Unavailable');
 const response=await binding.fetch(new Request('https://kortex-api.internal/api/v1/internal/sitemap?page='+page,{signal:AbortSignal.timeout(4000)}));
 if(!response.ok)throw Error('Unavailable');const text=await response.text();if(text.length>131072)throw Error('Oversize');
 const value=JSON.parse(text) as {items:{id:string;published_at:number}[];pages:number};
 if(!Array.isArray(value.items)||value.items.length>500||!Number.isInteger(value.pages)||value.pages<0||value.pages>100000||!value.items.every(t=>/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(t.id)&&t.id.length<=64&&Number.isSafeInteger(t.published_at)&&t.published_at>=0&&t.published_at<=8640000000000000))throw Error('Invalid sitemap');return value;
}
export const xmlResponse=(body:string)=>new Response('<?xml version="1.0" encoding="UTF-8"?>'+body,{headers:{'Content-Type':'application/xml; charset=utf-8','Cache-Control':'public, max-age=900','X-Content-Type-Options':'nosniff'}});
