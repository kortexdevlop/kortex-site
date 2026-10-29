import {test} from 'node:test';
import assert from 'node:assert/strict';
import {DatabaseSync} from 'node:sqlite';
import {readFileSync,readdirSync} from 'node:fs';
import {seedSql,statements} from './seed-10k.mjs';
test('bounded synthetic imports preserve embedded semicolons, schema invariants and previous batches',()=>{const db=new DatabaseSync(':memory:');try{for(const file of readdirSync('database/migrations').filter(f=>f.endsWith('.sql')).sort())db.exec(readFileSync('database/migrations/'+file,'utf8'));for(const [first,last]of [[1,500],[501,1000],[10001,10001]])for(const sql of statements(seedSql(first,last)))db.exec(sql);assert.equal(db.prepare("SELECT count(*) n FROM tools WHERE status='published'").get().n,1001);const contract=JSON.parse(db.prepare("SELECT contract_json FROM tool_versions WHERE tool_id='qa-tool-00500'").get().contract_json);assert.equal(contract.tool_id,'qa-tool-00500');assert(contract.actions.length>0);assert.equal(db.prepare("SELECT count(*) n FROM artifacts WHERE head_etag='SYNTHETIC_METADATA_NO_R2'").get().n,1001)}finally{db.close()}});
