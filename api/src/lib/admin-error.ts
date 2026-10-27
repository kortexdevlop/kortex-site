import type {ErrorCode} from './http';
export class AdminError extends Error {constructor(public status:number,public code:ErrorCode,message:string){super(message)}}
export const invalid=()=>new AdminError(422,'INTEGRITY_ERROR','Publication verification failed');
export const conflict=()=>new AdminError(409,'CONFLICT','Publication conflict');
export function canonical(value:unknown):string {
 if(Array.isArray(value))return '['+value.map(canonical).join(',')+']';
 if(value!==null&&typeof value==='object')return '{'+Object.keys(value).sort().map(k=>JSON.stringify(k)+':'+canonical((value as Record<string,unknown>)[k])).join(',')+'}';
 return JSON.stringify(value);
}
export const hex=(bytes:ArrayBuffer)=>Array.from(new Uint8Array(bytes),b=>b.toString(16).padStart(2,'0')).join('');
export async function readBounded(body:ReadableStream<Uint8Array>|null,max:number):Promise<Uint8Array> {
 if(!body)throw new AdminError(400,'BAD_REQUEST','Request body required');
 const reader=body.getReader(),chunks:Uint8Array[]=[];let length=0;
 try{for(;;){const r=await reader.read();if(r.done)break;length+=r.value.length;if(length>max){await reader.cancel();throw new AdminError(413,'BAD_REQUEST','Request body too large')}chunks.push(r.value)}}finally{reader.releaseLock()}
 const out=new Uint8Array(length);let at=0;for(const b of chunks){out.set(b,at);at+=b.length}return out;
}
