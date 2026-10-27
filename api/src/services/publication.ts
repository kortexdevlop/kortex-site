import type {Manifest,Contract} from '../../../shared/typescript/src/index';
import type {Env} from '../env';
import {PublicationRequest} from '../generated-validators';
import {AdminError,invalid,conflict,canonical,hex} from '../lib/admin-error';
import {extractArchive} from '../../../web-adapter/src/archive';
import {inspectWasm} from '../../../web-adapter/src/wasm-check';
import {MAX_TOOL_BYTES,checkedHead,type StoredArtifact} from './storage';
export interface Descriptor {schema_version:1;tool_id:string;version:string;abi_version:string;sha256:string;signing_key_id:string;signature:string}
export interface Publication {descriptor:Descriptor;manifest:Manifest;contract:Contract;r2_key:string;request_id:string}
export interface Actor {subject:string;jti:string}
interface Operation {request_sha256:string;operation:string;response_json:string}
export function publication(value:unknown):Publication {
 if(!PublicationRequest(value))throw new AdminError(400,'BAD_REQUEST','Invalid publication request');const p=structuredClone(value) as Publication;p.request_id=p.request_id.toLowerCase();const d=p.descriptor,m=p.manifest;
 if(d.tool_id!==m.id||d.version!==m.version||d.abi_version!==m.abi_version||p.contract.tool_id!==m.id||m.min_runtime!=='1.0.0'||p.r2_key!==`tools/${m.id}/${m.version}/${d.sha256}.tool`||p.contract.actions.some(a=>a.permissions.some(permission=>!m.permissions.includes(permission))))throw invalid();return p;
}
export async function fingerprint(value:unknown):Promise<string> {return hex(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(canonical(value))))}
async function replay(env:Env,request_id:string,hash:string,kind:string):Promise<unknown|null> {
 const old=await env.DB.prepare('SELECT request_sha256,operation,response_json FROM admin_operations WHERE request_id=?').bind(request_id).first<Operation>();
 if(!old)return null;if(old.request_sha256!==hash||old.operation!==kind)throw conflict();return JSON.parse(old.response_json);
}
function operation(env:Env,id:string,hash:string,kind:string,tool:string,version:string,actor:Actor,response:unknown,time:number,expected?:string) {
 return env.DB.prepare('INSERT INTO admin_operations(request_id,request_sha256,operation,tool_id,version,expected_current_version,subject,token_jti,response_json,created_at) VALUES(?,?,?,?,?,?,?,?,?,?)').bind(id,hash,kind,tool,version,expected??null,actor.subject,actor.jti,JSON.stringify(response),time);
}
async function commit(env:Env,statements:D1PreparedStatement[],id:string,hash:string,kind:string,response:unknown):Promise<unknown> {
 try{await env.DB.batch(statements);return response}catch(error){const previous=await replay(env,id,hash,kind);if(previous)return previous;
  if(/CONSTRAINT|CONFLICT|IMMUTABLE|REPLAY|ROLLBACK_UNAVAILABLE|EXPECTED_VERSION/.test(String(error)))throw conflict();throw error}
}
function standard64(text:string,length:number):Uint8Array {try{const bytes=Uint8Array.from(atob(text),c=>c.charCodeAt(0));if(bytes.length!==length||btoa(String.fromCharCode(...bytes))!==text)throw Error();return bytes}catch{throw invalid()}}
async function verifyPackage(env:Env,p:Publication,now:number) {
 const key=await env.DB.prepare('SELECT public_key_base64,not_before,expires,revoked FROM signing_keys WHERE id=?').bind(p.descriptor.signing_key_id).first<{public_key_base64:string;not_before:number;expires:number;revoked:number}>();
 if(!key||key.revoked||now<key.not_before||now>=key.expires)throw invalid();
 const publicKey=await crypto.subtle.importKey('raw',standard64(key.public_key_base64,32),{name:'Ed25519'},false,['verify']);
 const d=p.descriptor,text=`kortex-artifact-v1\n${d.tool_id}\n${d.version}\n${d.abi_version}\n${d.sha256}\n`;
 if(!await crypto.subtle.verify('Ed25519',publicKey,standard64(d.signature,64),new TextEncoder().encode(text)))throw invalid();
 const head=await env.TOOLS_BUCKET.head(p.r2_key);if(!head)throw new AdminError(404,'NOT_FOUND','Staged object not found');if(head.size<1||head.size>MAX_TOOL_BYTES)throw invalid();
 const object=await env.TOOLS_BUCKET.get(p.r2_key,{onlyIf:{etagMatches:head.etag}});if(!object||!('body' in object)||object.size!==head.size||object.etag!==head.etag)throw invalid();
 const bytes=new Uint8Array(head.size),reader=object.body.getReader(),digest=new crypto.DigestStream('SHA-256'),writer=digest.getWriter();let n=0;
 try{for(;;){const r=await reader.read();if(r.done)break;n+=r.value.length;if(n>head.size||n>MAX_TOOL_BYTES){await reader.cancel();throw invalid()}bytes.set(r.value,n-r.value.length);await writer.write(r.value)}await writer.close();if(n!==head.size||hex(await digest.digest)!==d.sha256)throw invalid()}
 catch(error){await writer.abort().catch(()=>{});await digest.digest.catch(()=>{});throw error}finally{reader.releaseLock();writer.releaseLock()}
 try {
  const files=await extractArchive(bytes),decode=(name:string)=>{const b=files.get(name);if(!b||b.length>1048576)throw invalid();return JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(b))};
  const names=[...files.keys()];if(names.join('\n')!==[...names].sort().join('\n'))throw invalid();
  const view=new DataView(bytes.buffer);let central=view.getUint32(bytes.length-6,true);
  for(let i=0;i<files.size;i++){const local=view.getUint32(central+42,true);if(view.getUint16(central+10,true)!==0||view.getUint16(central+12,true)!==0||view.getUint16(central+14,true)!==33||view.getUint32(central+38,true)!==0x81a40000||view.getUint16(local+10,true)!==0||view.getUint16(local+12,true)!==33)throw invalid();central+=46+view.getUint16(central+28,true)}
  if(canonical(decode('manifest.json'))!==canonical(p.manifest)||canonical(decode('contract.json'))!==canonical(p.contract))throw invalid();
  const allowed=new Set(['manifest.json','contract.json','module.wasm',p.manifest.icon_path]);
  if(!p.manifest.icon_path.startsWith('assets/')||!files.get(p.manifest.icon_path)?.length||files.size!==allowed.size||[...files.keys()].some(k=>!allowed.has(k)))throw invalid();
  const wasm=files.get('module.wasm')!;inspectWasm(wasm);if(!WebAssembly.validate(wasm))throw invalid();
 }catch{throw invalid()}
 return {size:n,etag:head.etag};
}
export async function publish(env:Env,value:unknown,actor:Actor):Promise<unknown> {
 const p=publication(value),hash=await fingerprint(p),kind='publish',old=await replay(env,p.request_id,hash,kind);if(old)return old;
 if(await env.DB.prepare('SELECT 1 FROM tool_versions WHERE tool_id=? AND version=?').bind(p.manifest.id,p.manifest.version).first())throw conflict();
 const time=Date.now(),checked=await verifyPackage(env,p,time),verifiedAt=Date.now(),m=p.manifest,c=p.contract,d=p.descriptor;
 const result={request_id:p.request_id,tool_id:m.id,version:m.version,status:'published'};
 const statements=[operation(env,p.request_id,hash,kind,m.id,m.version,actor,result,verifiedAt),
  env.DB.prepare('INSERT INTO categories(id,name) SELECT ?,? WHERE NOT EXISTS(SELECT 1 FROM categories WHERE id=?)').bind(m.category,m.category,m.category),
  env.DB.prepare('INSERT INTO tools(id,name,name_norm,summary,category_id,created_at,updated_at) SELECT ?,?,?,?,?,?,? WHERE NOT EXISTS(SELECT 1 FROM tools WHERE id=?)').bind(m.id,m.name,m.name.normalize('NFKC').toLowerCase(),m.summary,m.category,time,time,m.id),
  env.DB.prepare('INSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,created_at) VALUES(?,?,?,?,?,?,?,?)').bind(m.id,m.version,d.sha256,m.abi_version,m.ui_schema_version,canonical(m),canonical(c),time),
  ...c.actions.map(a=>env.DB.prepare('INSERT INTO actions(tool_id,version,action_id,description,input_schema_json,output_schema_json,permissions_json,side_effect,idempotent,timeout_ms) VALUES(?,?,?,?,?,?,?,?,?,?)').bind(m.id,m.version,a.id,a.description,canonical(a.input_schema),canonical(a.output_schema),canonical(a.permissions),a.side_effect,a.idempotent?1:0,a.timeout_ms)),
  env.DB.prepare('INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at) VALUES(?,?,?,?,?,?,?,?)').bind(m.id,m.version,d.sha256,p.r2_key,checked.size,d.signing_key_id,canonical(d),time),
  env.DB.prepare('UPDATE artifacts SET head_byte_length=?,head_etag=?,read_byte_length=?,read_sha256=?,signature_verified_at=?,r2_verified_at=? WHERE tool_id=? AND version=?').bind(checked.size,checked.etag,checked.size,d.sha256,verifiedAt,verifiedAt,m.id,m.version),
  env.DB.prepare('INSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at) VALUES(?,?,?,?,?,?)').bind(p.request_id,hash,m.id,m.version,time,verifiedAt),
  env.DB.prepare("UPDATE publication_jobs SET status='validated' WHERE request_id=?").bind(p.request_id),
  env.DB.prepare("UPDATE publication_jobs SET status='published' WHERE request_id=?").bind(p.request_id),
  env.DB.prepare("UPDATE tool_versions SET status='published' WHERE tool_id=? AND version=?").bind(m.id,m.version),
  env.DB.prepare("UPDATE tools SET name=?,name_norm=?,summary=?,category_id=?,status='published',published_version=?,published_at=?,updated_at=max(updated_at,?) WHERE id=?").bind(m.name,m.name.normalize('NFKC').toLowerCase(),m.summary,m.category,m.version,verifiedAt,verifiedAt,m.id)
 ];return commit(env,statements,p.request_id,hash,kind,result);
}
export async function rollback(env:Env,tool:string,body:{request_id:string;version:string;expected_current_version:string},actor:Actor):Promise<unknown> {
 const kind='rollback',hash=await fingerprint({tool,...body}),old=await replay(env,body.request_id,hash,kind);if(old)return old;
 const target=await env.DB.prepare('SELECT v.manifest_json,a.r2_key,a.byte_length,a.head_etag,a.sha256 FROM tool_versions v JOIN artifacts a ON a.tool_id=v.tool_id AND a.version=v.version WHERE v.tool_id=? AND v.version=?').bind(tool,body.version).first<{manifest_json:string}&StoredArtifact>();if(!target)throw new AdminError(404,'NOT_FOUND','Rollback target not found');
 if(!await checkedHead(env.TOOLS_BUCKET,target))throw new AdminError(409,'CONFLICT','Rollback object unavailable');
 const m=JSON.parse(target.manifest_json) as Manifest,time=Date.now(),result={request_id:body.request_id,tool_id:tool,version:body.version,status:'published'};
 return commit(env,[operation(env,body.request_id,hash,kind,tool,body.version,actor,result,time,body.expected_current_version),
  env.DB.prepare("UPDATE tools SET name=?,name_norm=?,summary=?,category_id=?,published_version=?,published_at=?,updated_at=max(updated_at,?) WHERE id=?").bind(m.name,m.name.normalize('NFKC').toLowerCase(),m.summary,m.category,body.version,time,time,tool),
  env.DB.prepare("INSERT INTO publication_events(request_id,tool_id,version,event_type,created_at,detail_json) VALUES(?,?,?,'rollback',?,?)").bind(body.request_id,tool,body.version,time,canonical({previous_version:body.expected_current_version,subject:actor.subject}))
 ],body.request_id,hash,kind,result);
}
export async function revoke(env:Env,digest:string,body:{request_id:string;reason:string},actor:Actor):Promise<unknown> {
 const kind='revoke',hash=await fingerprint({digest,...body}),old=await replay(env,body.request_id,hash,kind);if(old)return old;
 const rows=await env.DB.prepare('SELECT tool_id,version FROM artifacts WHERE sha256=?').bind(digest).all<{tool_id:string;version:string}>();if(rows.results.length!==1)throw new AdminError(rows.results.length?409:404,rows.results.length?'CONFLICT':'NOT_FOUND','Artifact not found');
 const {tool_id:tool,version}=rows.results[0],time=Date.now(),result={request_id:body.request_id,tool_id:tool,version,status:'revoked'};
 return commit(env,[operation(env,body.request_id,hash,kind,tool,version,actor,result,time),
  env.DB.prepare("UPDATE tools SET status='archived',published_version=NULL,updated_at=max(updated_at,?) WHERE id=? AND published_version=?").bind(time,tool,version),
  env.DB.prepare('INSERT INTO revocations(request_id,tool_id,version,reason,created_at) VALUES(?,?,?,?,?)').bind(body.request_id,tool,version,body.reason,time),
  env.DB.prepare("INSERT INTO publication_events(request_id,tool_id,version,event_type,created_at,detail_json) VALUES(?,?,?,'artifact-revoked',?,?)").bind(body.request_id,tool,version,time,canonical({reason:body.reason,subject:actor.subject}))
 ],body.request_id,hash,kind,result);
}
