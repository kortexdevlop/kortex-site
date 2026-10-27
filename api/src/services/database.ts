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
