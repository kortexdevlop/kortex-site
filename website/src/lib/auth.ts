import type {ServiceBinding} from './service-binding';
export type AccountView={id:string;email?:string;email_verified?:boolean;sync_enabled:boolean};
export async function accountView(request:Request,binding:ServiceBinding|undefined,origin:string|undefined):Promise<AccountView|null>{
 if(!origin||new URL(request.url).origin!==origin)throw Error('Account service unavailable');
 const cookies=(request.headers.get('Cookie')??'').split(';').map(s=>s.trim()).filter(s=>s.startsWith('__Host-kortex_session='));if(cookies.length!==1)return null;
 if(!binding)throw Error('Account service unavailable');
 const response=await binding.fetch(new Request('https://kortex-api.internal/api/v1/me',{headers:{Cookie:cookies[0],Accept:'application/json'},redirect:'manual'}));
 if(response.status===401||response.status===404)return null;if(!response.ok)throw Error('Account service unavailable');const data=await response.json() as {user:AccountView};if(!data.user?.id)throw Error('Account service unavailable');return {id:data.user.id,sync_enabled:!!data.user.sync_enabled,...(typeof data.user.email==='string'?{email:data.user.email,email_verified:!!data.user.email_verified}:{})};
}
