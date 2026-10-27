import {ApiClient} from './api-client.ts';
export function catalogQuery(input:URLSearchParams):URLSearchParams {
  const out=new URLSearchParams({limit:'24'});
  const q=(input.get('q')||'').replace(/[\u0000-\u001f\u007f]/g,'').trim().slice(0,100);
  if(q)out.set('q',q);
  const category=input.get('category');if(category&&/^[a-z0-9][a-z0-9-]{0,63}$/.test(category))out.set('category',category);
  if(input.get('sort')==='name')out.set('sort','name');
  const cursor=input.get('cursor');if(cursor&&cursor.length<=4096)out.set('cursor',cursor);
  return out;
}
export function nextPage(query:URLSearchParams,cursor:string|null):string|null {
  if(!cursor)return null;const next=new URLSearchParams(query);next.set('cursor',cursor);return '/tools?'+next;
}
export async function catalogue(client:ApiClient,input:URLSearchParams){const query=catalogQuery(input);return {query,...await client.listTools(query)};}
