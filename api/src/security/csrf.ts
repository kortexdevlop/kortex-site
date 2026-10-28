import type {Env} from '../env';
import {opaqueHash} from './sessions';
export const csrfToken=(env:Env,sessionToken:string)=>opaqueHash(env,'kortex-csrf-v1',sessionToken);
export function equal(a:string,b:string):boolean{if(a.length!==b.length)return false;let diff=0;for(let n=0;n<a.length;n++)diff|=a.charCodeAt(n)^b.charCodeAt(n);return diff===0}
export async function checkCsrf(request:Request,env:Env,sessionToken:string):Promise<boolean>{
 const token=request.headers.get('X-CSRF-Token');return request.headers.get('Origin')===env.PUBLIC_WEB_ORIGIN&&request.headers.get('Sec-Fetch-Site')!=='cross-site'&&!!token&&token.length===64&&equal(token,await csrfToken(env,sessionToken));
}
