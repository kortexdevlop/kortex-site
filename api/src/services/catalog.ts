import {ToolSummary,ToolDetail,ToolActions} from '../generated-validators';
import {BadQuery,slug,signCursor,readCursor,type Scope} from '../lib/cursor';
import {listRows,detailRow,downloadRow,type CatalogRow} from './database';
export class CatalogIntegrityError extends Error {}
export async function publishedActions(db:D1Database,ids:string[]){
 const result:{tool_id:string;action_id:string;description:string;input_schema:Record<string,unknown>}[]=[];
 for(const id of ids){const actions=await detail(db,id,true) as {version:string;actions:{id:string;description:string;input_schema:Record<string,unknown>}[]}|null;if(!actions)continue;const artifact=await downloadRow(db,id,actions.version);if(!artifact||artifact.revoked||!artifact.verified||!artifact.published||artifact.tool_status!=='published'||artifact.version_status!=='published')continue;
  for(const action of actions.actions){if(result.length>=20)break;result.push({tool_id:id,action_id:action.id,description:action.description.slice(0,160),input_schema:action.input_schema})}
 }
 return result;
}
export function query(params:URLSearchParams):{scope:Scope;cursor:string|null} {
 const allowed=new Set(['cursor','limit','q','category','tag','sort']);
 for(const name of params.keys())if(!allowed.has(name)||params.getAll(name).length!==1)throw new BadQuery();
 const limit=params.get('limit')??'20',sort=params.get('sort')??'recent';
 if(!/^(?:[1-9]|[1-9][0-9]|100)$/.test(limit)||!['recent','name'].includes(sort))throw new BadQuery();
 const rawQ=params.get('q')??'',category=params.get('category')??'',tag=params.get('tag')??'',cursor=params.get('cursor');
 if(rawQ.length>200||/[\u0000-\u001f\u007f]/.test(rawQ+tag)||category&&!slug.safeParse(category).success||tag.length>40||params.has('tag')&&!tag.trim()||params.has('category')&&!category||cursor!==null&&(!cursor||cursor.length>4096))throw new BadQuery();
 const q=rawQ.normalize('NFKC').trim().toLowerCase();if(q.length>200)throw new BadQuery();
 return {scope:{sort:sort as Scope['sort'],limit:Number(limit),q,category,tag},cursor};
}
function summary(row:CatalogRow) {
 const manifest=JSON.parse(row.manifest_json);
 const result={id:row.id,version:row.version,name:row.name,summary:row.summary,category:row.category_id,tags:manifest.tags,icon_path:manifest.icon_path,abi_version:row.abi_version,sha256:row.sha256,published_at:row.published_at};
 if(!ToolSummary(result)||!Number.isSafeInteger(row.published_at)||manifest.id!==row.id||manifest.version!==row.version||manifest.name!==row.name||manifest.summary!==row.summary||manifest.category!==row.category_id||manifest.abi_version!==row.abi_version)throw new CatalogIntegrityError();return result;
}
export async function list(db:D1Database,params:URLSearchParams,key:CryptoKey) {
 const {scope,cursor}=query(params);const after=cursor?await readCursor(cursor,scope,key):undefined;
 const rows=await listRows(db,scope,after),page=rows.slice(0,scope.limit),last=page.at(-1);
 try{return {items:page.map(summary),next_cursor:rows.length>scope.limit&&last?await signCursor(scope,{id:last.id,key:scope.sort==='recent'?last.published_at:last.name_norm},key):null}}
 catch(error){if(error instanceof CatalogIntegrityError)throw error;throw new CatalogIntegrityError()}
}
export async function detail(db:D1Database,id:string,actionsOnly:boolean) {
 const row=await detailRow(db,id);if(!row)return null;
 try {
  const contract=JSON.parse(row.contract_json);
  if(actionsOnly){const value={tool_id:row.id,version:row.version,actions:contract.actions};if(contract.tool_id!==row.id||!ToolActions(value))throw new CatalogIntegrityError();return value}
  const manifest=JSON.parse(row.manifest_json),descriptor=row.descriptor_json?JSON.parse(row.descriptor_json):null;
  const value={tool:summary(row),manifest,contract,descriptor};
  if(!ToolDetail(value)||contract.tool_id!==row.id||descriptor.tool_id!==row.id||descriptor.version!==row.version||descriptor.sha256!==row.sha256||descriptor.abi_version!==row.abi_version)throw new CatalogIntegrityError();
  return value;
 }catch{throw new CatalogIntegrityError()}
}
