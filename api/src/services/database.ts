import type {Position,Scope} from '../lib/cursor';
export interface CatalogRow {id:string;name:string;name_norm:string;summary:string;category_id:string;published_at:number;version:string;sha256:string;abi_version:string;manifest_json:string;contract_json:string;descriptor_json:string|null}
const columns='t.id,t.name,t.name_norm,t.summary,t.category_id,t.published_at,v.version,v.sha256,v.abi_version,v.manifest_json,v.contract_json';
const visible="t.status='published' AND v.status='published'";
const join='FROM tools t JOIN tool_versions v ON v.tool_id=t.id AND v.version=t.published_version';
export async function listRows(db:D1Database,scope:Scope,after?:Position):Promise<CatalogRow[]> {
 const clauses=[visible],values:(string|number)[]=[];
 if(scope.q){clauses.push('(instr(t.name_norm,?)>0 OR instr(lower(t.summary),?)>0)');values.push(scope.q,scope.q)}
 if(scope.category){clauses.push('t.category_id=?');values.push(scope.category)}
 if(scope.tag){clauses.push("EXISTS(SELECT 1 FROM json_each(v.manifest_json,'$.tags') tag WHERE tag.value=?)");values.push(scope.tag)}
 if(after){clauses.push(scope.sort==='recent'?'(t.published_at,t.id)<(?,?)':'(t.name_norm,t.id)>(?,?)');values.push(after.key,after.id)}
 // Only these two fixed SQL orderings may be selected; all request values are bound.
 const order=scope.sort==='recent'?'t.published_at DESC,t.id DESC':'t.name_norm ASC,t.id ASC';
 values.push(scope.limit+1);
 const result=await db.prepare(`SELECT ${columns} ${join} WHERE ${clauses.join(' AND ')} ORDER BY ${order} LIMIT ?`).bind(...values).all<CatalogRow>();
 if(!result.success)throw Error('DATABASE_UNAVAILABLE');return result.results;
}
export async function detailRow(db:D1Database,id:string):Promise<CatalogRow|null> {
 return db.prepare(`SELECT ${columns},a.descriptor_json ${join} LEFT JOIN artifacts a ON a.tool_id=v.tool_id AND a.version=v.version WHERE ${visible} AND t.id=?`).bind(id).first<CatalogRow>();
}
export interface DownloadRow {tool_id:string;version:string;sha256:string;r2_key:string;byte_length:number;head_etag:string;descriptor_json:string;tool_status:string;version_status:string;revoked:number;verified:number;published:number}
export async function downloadRow(db:D1Database,id:string,version:string):Promise<DownloadRow|null> {
 return db.prepare(`SELECT a.tool_id,a.version,a.sha256,a.r2_key,a.byte_length,a.head_etag,a.descriptor_json,t.status tool_status,v.status version_status,
 (k.revoked=1 OR v.status='revoked' OR EXISTS(SELECT 1 FROM revocations r WHERE (r.tool_id=a.tool_id AND r.version=a.version) OR r.signing_key_id=k.id)) revoked,
 (a.signature_verified_at IS NOT NULL AND a.r2_verified_at IS NOT NULL) verified,
 EXISTS(SELECT 1 FROM publication_jobs j WHERE j.tool_id=a.tool_id AND j.version=a.version AND j.status='published') published
 FROM artifacts a JOIN tools t ON t.id=a.tool_id JOIN tool_versions v ON v.tool_id=a.tool_id AND v.version=a.version JOIN signing_keys k ON k.id=a.signing_key_id
 WHERE a.tool_id=? AND a.version=?`).bind(id,version).first<DownloadRow>();
}
