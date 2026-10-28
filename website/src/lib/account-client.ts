export type Preferences={revision:number;sync_enabled:boolean;preferences:{theme:'light'|'dark';density:'simple'|'medium'|'pro';type:'standard'|'large';lang:'fr'|'en'}};
export type Library={revision:number;sync_enabled:boolean;items:{tool_id:string;tags:string[]}[];tombstones:{tool_id:string;tag:string;removed_revision:number}[]};
export type Operation={adds:{tool_id:string;tag:string}[];removes:{tool_id:string;tags:string[]}[]};
export class AccountError extends Error{constructor(public status:number,public limitSource:'application'|'unknown'='unknown'){super('Account request failed')}}
export async function accountRequest(path:string,init:RequestInit={}){const response=await fetch(path,{...init,credentials:'same-origin',cache:'no-store',redirect:'error',signal:AbortSignal.timeout(20000)});if(!response.ok)throw new AccountError(response.status,response.headers.get('X-Kortex-Limit-Source')==='application'?'application':'unknown');return response}
export async function accountSession(){const response=await accountRequest('/api/v1/me');return response.json() as Promise<{user:{id:string;sync_enabled:boolean};csrf_token:string}>}
export const accountHeaders=(owner:{user:{id:string};csrf_token:string},etag?:string)=>({'Content-Type':'application/json','X-CSRF-Token':owner.csrf_token,'X-Kortex-Account':owner.user.id,...(etag?{'If-Match':etag}:{})});
export async function libraryView(){return (await accountRequest('/api/v1/me/library')).json() as Promise<Library>}
export async function writeLibrary(owner:Awaited<ReturnType<typeof accountSession>>,state:Library,operation:Operation){
 const write=(revision:number)=>accountRequest('/api/v1/me/library',{method:'PUT',headers:accountHeaders(owner,`"library-${revision}"`),body:JSON.stringify(operation)});
 try{return await(await write(state.revision)).json() as Library}catch(error){if(!(error instanceof AccountError)||error.status!==409)throw error;const fresh=await libraryView();return await(await write(fresh.revision)).json() as Library}
}
