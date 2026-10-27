import type {Env} from '../env';
/** Reachable only through the named SitemapService binding, never the public router. */
export async function sitemap(request:Request,env:Env):Promise<Response>{
 const url=new URL(request.url),page=url.searchParams.get('page')||'0';
 if(url.pathname!=='/api/v1/internal/sitemap')return new Response(null,{status:404});
 if(!['GET','HEAD'].includes(request.method))return new Response(null,{status:405});
 if(!/^(0|[1-9][0-9]{0,4})$/.test(page)||url.searchParams.size!==1)return new Response(null,{status:400});
 try{
 const visible="FROM tools t JOIN tool_versions v ON v.tool_id=t.id AND v.version=t.published_version WHERE t.status='published' AND v.status='published'";
 const count=await env.DB.prepare(`SELECT count(*) total ${visible}`).first<number>('total');
 const rows=await env.DB.prepare(`SELECT t.id,t.published_at ${visible} ORDER BY t.id LIMIT 500 OFFSET ?`).bind(Number(page)*500).all<{id:string;published_at:number}>();
 if(!rows.success||count===null)throw Error('DB');
 return Response.json({items:rows.results,pages:Math.ceil(count/500)},{headers:{'Cache-Control':'no-store'}});
 }catch{return new Response(null,{status:503});}
}
