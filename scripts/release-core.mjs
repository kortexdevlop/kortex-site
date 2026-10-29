import {createHash,createPublicKey,verify} from 'node:crypto';
import {appendFileSync,mkdirSync,readFileSync,existsSync} from 'node:fs';
import {dirname} from 'node:path';
import {validatePublication} from './release-tool.mjs';
import {revocationText} from '../shared/typescript/src/revocation.ts';
export const canonical=value=>JSON.stringify(value,(_key,item)=>item&&typeof item==='object'&&!Array.isArray(item)?Object.fromEntries(Object.entries(item).sort(([a],[b])=>a<b?-1:a>b?1:0)):item);
export const hash=value=>createHash('sha256').update(typeof value==='string'||Buffer.isBuffer(value)?value:canonical(value)).digest('hex');
export function releasePlan(p){
 if(!p||p.schema_version!==1||!['staging','production','isolated-test'].includes(p.channel)||!['promote','rollback','revoke'].includes(p.action)||!/^[a-f0-9]{8}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{4}-[a-f0-9]{12}$/.test(p.request_id||''))throw Error('INVALID_RELEASE_PLAN');
 const extra=p.action==='promote'?['publication','archive_path']:p.action==='revoke'?['tool_id','version','sha256','reason','revocations']:p.target==='tool'?['target','tool_id','version','expected_current_version']:['target','version_id','expected_version_id','channels'];
 if(Object.keys(p).some(k=>!['schema_version','channel','action','request_id',...extra].includes(k)))throw Error('UNKNOWN_RELEASE_FIELD');
 if(p.action==='promote'){validatePublication(p.publication);if(p.publication.request_id!==p.request_id)throw Error('RELEASE_ID_MISMATCH')}
 else if(p.action==='revoke'||p.target==='tool'){if(!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(p.tool_id||'')||!/^\d+\.\d+\.\d+$/.test(p.version||''))throw Error('INVALID_RELEASE_TARGET');if(p.action==='revoke'&&(!/^[a-f0-9]{64}$/.test(p.sha256||'')||typeof p.reason!=='string'||!p.reason.trim()||p.reason.length>1000))throw Error('INVALID_REVOCATION');if(p.target==='tool'&&!/^\d+\.\d+\.\d+$/.test(p.expected_current_version||''))throw Error('EXPECTED_VERSION_REQUIRED')}
 else if(!['api','website','runtime','desktop'].includes(p.target)||p.target!=='desktop'&&(!/^[a-f0-9-]{36}$/.test(p.version_id||'')||!/^[a-f0-9-]{36}$/.test(p.expected_version_id||'')))throw Error('INVALID_HOST_ROLLBACK');return p;
}
export function verifyRevocationUpdate(next,old,root,target,now=Date.now()){
 const text=revocationText(next);if(next.root_key_id!==root.id||next.version<=old.version||next.issued_at>now||next.valid_until<=now||old.revoked_keys.some(k=>!next.revoked_keys.includes(k))||old.revoked_artifacts.some(a=>!next.revoked_artifacts.some(b=>canonical(a)===canonical(b)))||!next.revoked_artifacts.some(a=>a.tool_id===target.tool_id&&a.version===target.version&&a.sha256===target.sha256))throw Error('REVOCATION_UPDATE_UNSAFE');
 const key=createPublicKey({key:Buffer.concat([Buffer.from('302a300506032b6570032100','hex'),Buffer.from(root.public_key_base64,'base64')]),format:'der',type:'spki'});if(!verify(null,text,key,Buffer.from(next.signature,'base64')))throw Error('REVOCATION_SIGNATURE');return next;
}
export class ReleaseJournal{
 constructor(path){this.path=path;this.last='0'.repeat(64);this.sequence=0;mkdirSync(dirname(path),{recursive:true});if(existsSync(path))for(const line of readFileSync(path,'utf8').trim().split('\n').filter(Boolean)){const row=JSON.parse(line),{sha256,...body}=row;if(row.sequence!==this.sequence+1||row.previous!==this.last||hash(body)!==sha256)throw Error('RELEASE_JOURNAL_TAMPERED');this.sequence=row.sequence;this.last=sha256}}
 append(request_id,event,detail={}){const body={sequence:++this.sequence,previous:this.last,date:new Date().toISOString(),request_id,event,detail};const row={...body,sha256:hash(body)};appendFileSync(this.path,canonical(row)+'\n',{flag:'a'});this.last=row.sha256;return row}
}
export async function executeRelease(plan,io,{fault=async()=>{}}={}){
 releasePlan(plan);if(plan.channel==='production')throw Error('PRODUCTION_RELEASE_GATES_OPEN');if(plan.channel!==io.channel)throw Error('RELEASE_CHANNEL_MISMATCH');
 const record=(event,detail={})=>io.record(plan.request_id,event,detail);await record('started',{action:plan.action,fingerprint:hash(plan)});
 const old=await io.operation(plan);if(old){await record('reconciled',{action:plan.action});return{...old,reconciled:true}}
 await io.backup(plan,hash(plan));await record('backed_up');
 try{
  if(plan.action==='promote'){await io.upload(plan.publication);await record('uploaded',{sha256:plan.publication.descriptor.sha256});await fault('before_publish');const result=await io.admin('publications',plan.publication);await fault('after_publish');const current=await io.detail(plan.publication.manifest.id);if(current?.descriptor?.sha256!==plan.publication.descriptor.sha256)throw Error('PUBLICATION_NOT_VISIBLE');await record('committed',{sha256:current.descriptor.sha256});return result}
  if(plan.action==='revoke'){await io.revocationDocument(plan);await record('revocations_installed');await fault('before_revoke');const result=await io.admin('artifacts/'+plan.sha256+'/revoke',{request_id:plan.request_id,reason:plan.reason});await fault('after_revoke');if((await io.detail(plan.tool_id))?.descriptor?.sha256===plan.sha256)throw Error('REVOCATION_NOT_VISIBLE');await record('committed',{sha256:plan.sha256});return result}
  if(plan.target==='tool'){const result=await io.admin('tools/'+plan.tool_id+'/rollback',{request_id:plan.request_id,version:plan.version,expected_current_version:plan.expected_current_version});await fault('after_rollback');const current=await io.detail(plan.tool_id);if(current?.tool?.version!==plan.version)throw Error('ROLLBACK_NOT_VISIBLE');await record('committed',{version:plan.version});return result}
  const result=await io.hostRollback(plan);await record('committed',{target:plan.target});return result;
 }catch(e){await record('interrupted',{action:plan.action,code:/^[A-Z0-9_]+$/.test(e.message)?e.message:'TRANSPORT_OR_SERVICE_FAILURE'});throw e}
}
