import {test} from 'node:test';import assert from 'node:assert/strict';import {mkdtempSync,writeFileSync,readFileSync} from 'node:fs';import {tmpdir} from 'node:os';import {join} from 'node:path';import {spawnSync} from 'node:child_process';
test('real alert evaluator detects failure and recovery and never exports private log fields',()=>{
 const dir=mkdtempSync(join(tmpdir(),'kortex-alert-')),source=join(dir,'events.ndjson'),minute=Math.floor(Date.now()/60000);
 const run=errors=>{writeFileSync(source,Array.from({length:100},(_,n)=>JSON.stringify({v:1,event:'request',minute,status:n<errors?503:200,duration_ms:12,private:'PRIVATE_FILE_photo.jpg email@example.test'})).join('\n'));return spawnSync(process.execPath,['scripts/health-summary.mjs',source],{encoding:'utf8',windowsHide:true})};
 const failed=run(3);assert.equal(failed.status,2);const alert=JSON.parse(failed.stdout);assert.equal(alert.alert,true);assert.equal(alert.notification_sent,false);assert.ok(!failed.stdout.includes('PRIVATE_FILE'));assert.ok(!failed.stdout.includes('email@example.test'));
 const recovered=run(0);assert.equal(recovered.status,0);assert.equal(JSON.parse(recovered.stdout).alert,false);
});
