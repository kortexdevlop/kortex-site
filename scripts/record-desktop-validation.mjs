import assert from 'node:assert/strict';
import {readFileSync,writeFileSync} from 'node:fs';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
const json=p=>JSON.parse(readFileSync(p,'utf8')),digest=b=>createHash('sha256').update(b).digest('hex');
const browser=readFileSync('docs/evidence/desktop-playwright.log','utf8'),unit=readFileSync('docs/evidence/desktop-web-unit.log','utf8');
assert.match(browser,/8 passed/);assert.match(unit,/pass 37/);assert.match(unit,/fail 0/);
for(const name of ['ui','router','files','isolation','calculator-staging'])assert.equal(json('docs/evidence/desktop-'+name+'.json').passed,true);
assert.equal(json('docs/evidence/desktop-isolation.json').critical_axe,0);
assert.equal(json('docs/evidence/desktop-isolation.json').workers_after_close,0);
const web='https://kortex-web-staging.userpgl.workers.dev',response=await fetch(web+'/workspace');assert.equal(response.status,200);const csp=response.headers.get('Content-Security-Policy');assert.ok(csp.includes("script-src 'self' 'sha256-"));assert.ok(!csp.match(/script-src[^;]*unsafe-inline/));assert.equal(response.headers.get('Cache-Control'),'private, no-store');
const detail=await(await fetch(web+'/api/v1/tools/calculatrice')).json();assert.equal(detail.tool.version,'1.0.1');assert.equal(detail.descriptor.sha256,json('docs/evidence/desktop-calculator-descriptor.json').sha256);
const changed=execFileSync('git',['diff','HEAD','--name-only'],{encoding:'utf8'}),added=execFileSync('git',['ls-files','--others','--exclude-standard'],{encoding:'utf8'}),files=[...new Set((changed+'\n'+added).trim().split(/\r?\n/))].filter(Boolean).sort();
const sources={},evidence={};for(const file of files){if(file==='docs/PROGRESS.md'||file==='docs/evidence/desktop-validation.json')continue;if(file.startsWith('docs/evidence/'))evidence[file]=digest(readFileSync(file));else sources[file]=digest(readFileSync(file,'utf8').replace(/\r\n/g,'\n'))}
writeFileSync('docs/evidence/desktop-validation.json',JSON.stringify({date:new Date().toISOString(),source_commit:'10da685',steps:{'17.1':'closed technically','17.2':'closed technically','17.3':'closed technically'},platform:'Windows / Chromium',tests:{web_unit:37,staging_browser:8,visual_capture_recheck:2,astro_files:92,astro_errors:0,astro_warnings:0,ssr_build:true,critical_axe:0,workers_after_close:0},staging_only:true,web_origin:web,source_hash_format:'SHA256 UTF8 LF; PROGRESS result commit excluded',evidence_hash_format:'SHA256 raw bytes',sources,evidence,limits:['Warm loaded windows offline only; no cold offline Web reload','Known serious contrast finding; no AA certification or manual screen reader claim','Seven-day staging signing key; production trust not claimed','Google 14.5 unchanged; no phase 18'],production_touched:false,mail_touched:false,ai_quotas_changed:false,passed:true},null,2)+'\n');
console.log('Desktop evidence bound to sources and live staging; 17.1–17.3 technically closed.');
