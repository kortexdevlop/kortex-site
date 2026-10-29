export type RevokedArtifact={tool_id:string;version:string;sha256:string};
export type RevocationDocument={schema_version:1;version:number;issued_at:number;valid_until:number;root_key_id:string;revoked_keys:string[];revoked_artifacts:RevokedArtifact[];signature:string};
export const REVOCATION_TTL=86400000;
const exact=(v:unknown,keys:string[]):v is Record<string,any>=>!!v&&typeof v==='object'&&!Array.isArray(v)&&Object.keys(v).sort().join('|')===[...keys].sort().join('|');
const id=(v:unknown):v is string=>typeof v==='string'&&/^[A-Za-z0-9_-]{1,64}$/.test(v);
const integer=(v:unknown):v is number=>typeof v==='number'&&Number.isSafeInteger(v)&&v>=0;
export function revocationText(value:unknown):Uint8Array<ArrayBuffer>{
 if(!exact(value,['schema_version','version','issued_at','valid_until','root_key_id','revoked_keys','revoked_artifacts','signature'])||value.schema_version!==1||!integer(value.version)||value.version<1||!integer(value.issued_at)||!integer(value.valid_until)||value.valid_until<=value.issued_at||value.valid_until-value.issued_at>REVOCATION_TTL||!id(value.root_key_id)||!Array.isArray(value.revoked_keys)||value.revoked_keys.length>256||!Array.isArray(value.revoked_artifacts)||value.revoked_artifacts.length>4096||typeof value.signature!=='string'||value.signature.length>128)throw Error('INVALID_REVOCATIONS');
 const keys=value.revoked_keys.map((k:unknown)=>{if(!id(k))throw Error('INVALID_REVOCATIONS');return k}).sort();if(new Set(keys).size!==keys.length)throw Error('INVALID_REVOCATIONS');
 const artifacts=value.revoked_artifacts.map((a:unknown)=>{if(!exact(a,['tool_id','version','sha256'])||typeof a.tool_id!=='string'||!/^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(a.tool_id)||a.tool_id.length>64||typeof a.version!=='string'||!/^[0-9A-Za-z.+-]{5,128}$/.test(a.version)||typeof a.sha256!=='string'||!/^[a-f0-9]{64}$/.test(a.sha256))throw Error('INVALID_REVOCATIONS');return `${a.tool_id}|${a.version}|${a.sha256}`}).sort();if(new Set(artifacts).size!==artifacts.length)throw Error('INVALID_REVOCATIONS');
 return new Uint8Array(new TextEncoder().encode(`kortex-revocations-v1\n${value.version}\n${value.issued_at}\n${value.valid_until}\n${value.root_key_id}\n${keys.map(k=>'key|'+k+'\n').join('')}${artifacts.map(a=>'artifact|'+a+'\n').join('')}`));
}
export function isRevoked(doc:RevocationDocument,proof:{tool_id:string;version:string;sha256:string;signing_key_id:string}){return doc.revoked_keys.includes(proof.signing_key_id)||doc.revoked_artifacts.some(a=>a.tool_id===proof.tool_id&&a.version===proof.version&&a.sha256===proof.sha256)}
