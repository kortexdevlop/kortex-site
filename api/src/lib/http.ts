export type ErrorCode='BAD_REQUEST'|'NOT_FOUND'|'CONFLICT'|'UNAUTHORIZED'|'FORBIDDEN'|'RATE_LIMITED'|'UNAVAILABLE'|'INTEGRITY_ERROR';
export const uuid=/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
export function requestId(request:Request):string {
 const id=request.headers.get('x-request-id');return id&&uuid.test(id)?id:crypto.randomUUID();
}
export function json(value:unknown,status:number,id:string,origin?:string,head=false,extra?:HeadersInit):Response {
 const headers=new Headers(extra);
 headers.set('Content-Type','application/json; charset=utf-8');headers.set('Cache-Control','no-store');
 headers.set('Content-Security-Policy',"default-src 'none'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'");
 headers.set('X-Content-Type-Options','nosniff');headers.set('Referrer-Policy','no-referrer');headers.set('X-Request-ID',id);headers.set('Vary','Origin');
 if(origin)headers.set('Access-Control-Allow-Origin',origin);
 return new Response(head||status===204?null:JSON.stringify(value),{status,headers});
}
export function failure(code:ErrorCode,message:string,status:number,id:string,origin?:string,head=false,extra?:HeadersInit):Response {
 return json({error:{code,message,request_id:id}},status,id,origin,head,extra);
}
