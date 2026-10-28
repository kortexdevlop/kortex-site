import {z} from 'zod';
import {AdminError} from '../lib/admin-error';
export const defaults={theme:'light',density:'medium',type:'standard',lang:'fr'} as const;
export const preferencesSchema=z.object({theme:z.enum(['light','dark']).optional(),density:z.enum(['simple','medium','pro']).optional(),type:z.enum(['standard','large']).optional(),lang:z.enum(['fr','en']).optional()}).strict();
const slug=z.string().min(3).max(64).regex(/^[a-z0-9]+(?:-[a-z0-9]+)*$/),tag=z.uuid();
export const librarySchema=z.object({adds:z.array(z.object({tool_id:slug,tag}).strict()).max(100).default([]),removes:z.array(z.object({tool_id:slug,tags:z.array(tag).min(1).max(100)}).strict()).max(100).default([])}).strict();
export const settingsSchema=z.object({sync_enabled:z.boolean().optional(),preferences:preferencesSchema.optional()}).strict().refine(v=>v.sync_enabled!==undefined||v.preferences!==undefined);
export type LibraryInput=z.infer<typeof librarySchema>;
type State={library_revision:number;preferences_revision:number;sync_enabled:number};
type RecordRow={tool_id:string;tag:string;added_revision:number;removed_revision:number|null};
export class SyncError extends AdminError{}
export const etag=(entity:'library'|'preferences',revision:number)=>`"${entity}-${revision}"`;
export function revision(header:string|null,entity:'library'|'preferences'):number{
 const match=new RegExp('^"'+entity+'-(0|[1-9][0-9]{0,15})"$').exec(header??'');const value=match?Number(match[1]):NaN;
 if(!Number.isSafeInteger(value)||value<0||value>=9007199254740990)throw new SyncError(409,'CONFLICT','A current revision is required');return value;
}
export async function ensureSync(db:D1Database,user:string){await db.batch([db.prepare('INSERT INTO user_sync(user_id) SELECT id FROM users WHERE id=? ON CONFLICT(user_id) DO NOTHING').bind(user),db.prepare('INSERT INTO user_preferences(user_id,data_json) SELECT id,? FROM users WHERE id=? ON CONFLICT(user_id) DO NOTHING').bind(JSON.stringify(defaults),user)])}
export async function readLibrary(db:D1Database,user:string){
 await ensureSync(db,user);const rows=await db.batch([db.prepare('SELECT s.library_revision,s.preferences_revision,u.sync_enabled FROM user_sync s JOIN users u ON u.id=s.user_id WHERE s.user_id=?').bind(user),db.prepare('SELECT tool_id,tag,added_revision,removed_revision FROM user_library WHERE user_id=? ORDER BY tool_id,tag').bind(user)]);
 const state=rows[0].results[0] as State|undefined;if(!state)throw new SyncError(401,'UNAUTHORIZED','Session unavailable');
 const records=rows[1].results as RecordRow[],items=new Map<string,string[]>();for(const row of records)if(row.removed_revision===null){const tags=items.get(row.tool_id)??[];tags.push(row.tag);items.set(row.tool_id,tags)}
 return {revision:state.library_revision,sync_enabled:!!state.sync_enabled,items:[...items].map(([tool_id,tags])=>({tool_id,tags})),tombstones:records.filter(r=>r.removed_revision!==null).map(({tool_id,tag,removed_revision})=>({tool_id,tag,removed_revision})),records};
}
export async function readPreferences(db:D1Database,user:string){
 await ensureSync(db,user);const row=await db.prepare('SELECT s.preferences_revision AS revision,u.sync_enabled,p.data_json FROM user_sync s JOIN users u ON u.id=s.user_id JOIN user_preferences p ON p.user_id=s.user_id WHERE s.user_id=?').bind(user).first<{revision:number;sync_enabled:number;data_json:string}>();if(!row)throw new SyncError(401,'UNAUTHORIZED','Session unavailable');return {revision:row.revision,sync_enabled:!!row.sync_enabled,preferences:JSON.parse(row.data_json) as typeof defaults};
}
export async function mutateLibrary(db:D1Database,user:string,expected:number,input:LibraryInput){
 const snapshot=await readLibrary(db,user);if(!snapshot.sync_enabled)throw new SyncError(403,'FORBIDDEN','Synchronization is disabled');if(snapshot.revision!==expected)throw new SyncError(409,'CONFLICT','Library changed; refresh before retrying');
 const adds=input.adds,removes=input.removes.flatMap(r=>r.tags.map(tag=>({tool_id:r.tool_id,tag}))),all=[...adds,...removes],seen=new Map<string,string>();if(all.length<1||all.length>100)throw new SyncError(400,'BAD_REQUEST','Use between 1 and 100 tag operations');
 for(const row of [...snapshot.records,...all]){if(seen.has(row.tag)&&seen.get(row.tag)!==row.tool_id)throw new SyncError(409,'CONFLICT','A tag cannot identify two tools');seen.set(row.tag,row.tool_id)}if(seen.size>2000)throw new SyncError(413,'BAD_REQUEST','Library metadata quota reached');
 const token=crypto.randomUUID(),addJson=JSON.stringify(adds),removeJson=JSON.stringify(removes),allJson=JSON.stringify(all),now=Date.now();
 const results=await db.batch([
 db.prepare(`UPDATE user_sync SET library_revision=library_revision+1,library_mutation=? WHERE user_id=? AND library_revision=? AND library_revision<9007199254740990 AND EXISTS(SELECT 1 FROM users WHERE id=? AND sync_enabled=1)
 AND (SELECT count(*) FROM user_library WHERE user_id=?) + (SELECT count(*) FROM (SELECT DISTINCT json_extract(value,'$.tag') AS tag FROM json_each(?)) n WHERE NOT EXISTS(SELECT 1 FROM user_library l WHERE l.user_id=? AND l.tag=n.tag))<=2000
 AND NOT EXISTS(SELECT 1 FROM json_each(?) j JOIN user_library l ON l.user_id=? AND l.tag=json_extract(j.value,'$.tag') WHERE l.tool_id!=json_extract(j.value,'$.tool_id'))`).bind(token,user,expected,user,user,allJson,user,allJson,user),
 db.prepare(`INSERT INTO user_library SELECT s.user_id,json_extract(j.value,'$.tool_id'),json_extract(j.value,'$.tag'),s.library_revision,NULL FROM user_sync s,json_each(?) j WHERE s.user_id=? AND s.library_mutation=? ON CONFLICT(user_id,tag) DO NOTHING`).bind(addJson,user,token),
 db.prepare(`INSERT INTO user_library SELECT s.user_id,json_extract(j.value,'$.tool_id'),json_extract(j.value,'$.tag'),0,s.library_revision FROM user_sync s,json_each(?) j WHERE s.user_id=? AND s.library_mutation=? ON CONFLICT(user_id,tag) DO UPDATE SET removed_revision=coalesce(user_library.removed_revision,excluded.removed_revision)`).bind(removeJson,user,token),
 db.prepare("INSERT INTO sync_events SELECT user_id,'library',library_revision,? FROM user_sync WHERE user_id=? AND library_mutation=?").bind(now,user,token),
 db.prepare("DELETE FROM sync_events WHERE user_id=? AND entity='library' AND revision<(SELECT library_revision-99 FROM user_sync WHERE user_id=? AND library_mutation=?)").bind(user,user,token)]);
 if(results[0].meta.changes!==1)throw new SyncError(409,'CONFLICT','Library changed; refresh before retrying');return readLibrary(db,user);
}
export async function mutatePreferences(db:D1Database,user:string,expected:number,input:z.infer<typeof settingsSchema>){
 const snapshot=await readPreferences(db,user);if(snapshot.revision!==expected)throw new SyncError(409,'CONFLICT','Preferences changed; refresh before retrying');const enabled=input.sync_enabled??snapshot.sync_enabled;
 if(input.preferences&&!enabled)throw new SyncError(403,'FORBIDDEN','Enable synchronization before uploading preferences');
 const data={...snapshot.preferences,...input.preferences},token=crypto.randomUUID(),now=Date.now();
 const results=await db.batch([db.prepare('UPDATE user_sync SET preferences_revision=preferences_revision+1,preferences_mutation=? WHERE user_id=? AND preferences_revision=? AND preferences_revision<9007199254740990').bind(token,user,expected),db.prepare('UPDATE users SET sync_enabled=? WHERE id=? AND EXISTS(SELECT 1 FROM user_sync WHERE user_id=? AND preferences_mutation=?)').bind(Number(enabled),user,user,token),db.prepare('UPDATE user_preferences SET data_json=? WHERE user_id=? AND EXISTS(SELECT 1 FROM user_sync WHERE user_id=? AND preferences_mutation=?)').bind(JSON.stringify(data),user,user,token),db.prepare("INSERT INTO sync_events SELECT user_id,'preferences',preferences_revision,? FROM user_sync WHERE user_id=? AND preferences_mutation=?").bind(now,user,token),db.prepare("DELETE FROM sync_events WHERE user_id=? AND entity='preferences' AND revision<(SELECT preferences_revision-99 FROM user_sync WHERE user_id=? AND preferences_mutation=?)").bind(user,user,token)]);
 if(results[0].meta.changes!==1)throw new SyncError(409,'CONFLICT','Preferences changed; refresh before retrying');return readPreferences(db,user);
}
