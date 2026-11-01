import type {Env} from '../env';
import {json,failure} from '../lib/http';
import {revocationText,type RevocationDocument} from '../../../shared/typescript/src/revocation';
import {trustSchedule} from '../services/trust-schedule';
export async function revocations(request:Request,env:Env,id:string,origin?:string):Promise<Response|null>{
 const url=new URL(request.url);if(url.pathname!=='/api/v1/trust/revocations')return null;if(!['GET','HEAD'].includes(request.method)||url.search)return failure('BAD_REQUEST','Invalid trust request',400,id,origin);
 try{const raw=env.REVOCATION_DOCUMENT,now=Date.now(),own=raw&&raw.length<=1048576?[JSON.parse(raw) as RevocationDocument]:[],scheduled=(await trustSchedule(env)).revocations as RevocationDocument[];const d=[...own,...scheduled].filter(c=>now>=c.issued_at&&now<c.valid_until).sort((a,b)=>b.version-a.version)[0],root=JSON.parse(env.REVOCATION_ROOT??'null');if(!d)throw Error();const bytes=revocationText(d);if(!root||root.id!==d.root_key_id)throw Error();const key=await crypto.subtle.importKey('raw',Uint8Array.from(atob(root.public_key_base64),c=>c.charCodeAt(0)),{name:'Ed25519'},false,['verify']);if(!await crypto.subtle.verify('Ed25519',key,Uint8Array.from(atob(d.signature),c=>c.charCodeAt(0)),bytes))throw Error();
 const keys=await env.DB.prepare('SELECT id FROM signing_keys WHERE revoked=1 LIMIT 257').all<{id:string}>(),rows=await env.DB.prepare('SELECT a.tool_id,a.version,a.sha256 FROM artifacts a WHERE EXISTS(SELECT 1 FROM revocations r WHERE r.tool_id=a.tool_id AND r.version=a.version) OR EXISTS(SELECT 1 FROM tool_versions v WHERE v.tool_id=a.tool_id AND v.version=a.version AND v.status=\'revoked\') LIMIT 4097').all<{tool_id:string;version:string;sha256:string}>();
 if(keys.results.some(k=>!d.revoked_keys.includes(k.id))||rows.results.some(a=>!d.revoked_artifacts.some(b=>a.tool_id===b.tool_id&&a.version===b.version&&a.sha256===b.sha256)))throw Error();return json(d,200,id,origin,request.method==='HEAD',{'Cache-Control':'no-store'});
 }catch{return failure('UNAVAILABLE','Signed revocations unavailable; reconnect after operator refresh',503,id,origin,request.method==='HEAD')}
}
