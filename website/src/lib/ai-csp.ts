/** Hash only inline scripts in the trusted, freshly rendered assistant HTML (Astro hydration bootstrap). */
export async function assistantCsp(response:Response):Promise<Response>{
 const html=await response.text();if(new TextEncoder().encode(html).length>1048576)return new Response('Assistant indisponible',{status:503});
 const hashes:string[]=[];for(const match of html.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/gi)){if(/\bsrc\s*=/i.test(match[1])||!/\S/.test(match[2])||/type=["']application\//i.test(match[1]))continue;const hash=new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(match[2])));hashes.push("'sha256-"+btoa(String.fromCharCode(...hash))+"'")}
 const headers=new Headers(response.headers),policy=headers.get('Content-Security-Policy');if(policy)headers.set('Content-Security-Policy',policy.replace("script-src 'self'","script-src 'self' "+hashes.join(' ')));headers.delete('Content-Length');return new Response(html,{status:response.status,headers});
}
