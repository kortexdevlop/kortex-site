import {env} from 'cloudflare:workers';
import {applyD1Migrations} from 'cloudflare:test';
export const hex=(b:ArrayBuffer)=>Array.from(new Uint8Array(b),x=>x.toString(16).padStart(2,'0')).join('');
export async function seedArtifact(id:string,bytes:Uint8Array,version='1.0.0') {
 await applyD1Migrations(env.DB,env.TEST_MIGRATIONS);
 const sha256=hex(await crypto.subtle.digest('SHA-256',bytes)),key='tools/'+id+'/'+version+'/'+sha256+'.tool';
 const object=await env.TOOLS_BUCKET.put(key,bytes);if(!object)throw Error('PUT_FAILED');
 const m:Record<string,unknown>={...env.TEST_MANIFEST,id,version,category:'testcat'},c={...env.TEST_CONTRACT,tool_id:id};
 const d={...env.TEST_DESCRIPTOR,tool_id:id,version,sha256,signing_key_id:'download-test'};
 const time=Date.now();
 await env.DB.batch([
  env.DB.prepare("INSERT INTO categories VALUES('testcat','Test category') ON CONFLICT DO NOTHING"),
  env.DB.prepare("INSERT INTO signing_keys SELECT 'download-test',?,1,0,9999999999999,0 WHERE NOT EXISTS(SELECT 1 FROM signing_keys WHERE id='download-test')").bind('A'.repeat(43)+'='),
  env.DB.prepare('INSERT INTO tools(id,name,name_norm,summary,category_id,created_at,updated_at) VALUES(?,?,?,?,?,?,?)').bind(id,m.name,String(m.name).normalize('NFKC').toLowerCase(),m.summary,m.category,time,time),
  env.DB.prepare("INSERT INTO tool_versions(tool_id,version,sha256,abi_version,ui_schema_version,manifest_json,contract_json,created_at) VALUES(?,?,?,'1.0','1.0',?,?,?)").bind(id,version,sha256,JSON.stringify(m),JSON.stringify(c),time),
  env.DB.prepare('INSERT INTO artifacts(tool_id,version,sha256,r2_key,byte_length,signing_key_id,descriptor_json,created_at) VALUES(?,?,?,?,?,?,?,?)').bind(id,version,sha256,key,bytes.length,'download-test',JSON.stringify(d),time),
  env.DB.prepare('UPDATE artifacts SET head_byte_length=byte_length,head_etag=?,read_byte_length=byte_length,read_sha256=sha256,signature_verified_at=?,r2_verified_at=? WHERE tool_id=?').bind(object.etag,time,time,id),
  env.DB.prepare('INSERT INTO publication_jobs(request_id,request_sha256,tool_id,version,created_at,updated_at) VALUES(?,?,?,?,?,?)').bind(crypto.randomUUID(),sha256,id,version,time,time),
  env.DB.prepare("UPDATE publication_jobs SET status='validated' WHERE tool_id=?").bind(id),
  env.DB.prepare("UPDATE publication_jobs SET status='published' WHERE tool_id=?").bind(id),
  env.DB.prepare("UPDATE tool_versions SET status='published' WHERE tool_id=?").bind(id),
  env.DB.prepare("UPDATE tools SET status='published',published_version=?,published_at=? WHERE id=?").bind(version,time,id)
 ]);
 return {key,sha256,object,descriptor:d};
}
