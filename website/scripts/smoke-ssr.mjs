import assert from 'node:assert/strict';
import {writeFileSync} from 'node:fs';
const origin=new URL(process.argv[2]??'http://127.0.0.1:4330');
if(origin.protocol!=='http:'||origin.hostname!=='127.0.0.1'||origin.pathname!=='/')throw Error('Local loopback server required');
const checks=[];
async function check(path,status,init){const response=await fetch(new URL(path,origin),init);assert.equal(response.status,status,path);const text=await response.text();checks.push({path,status,method:init?.method??'GET'});return {response,text};}
const home=await check('/',503);assert.match(home.text,/momentanément indisponible/);assert.doesNotMatch(home.text,/demo-visual-|reference-workspace-|photos-vacances\.zip|Exemple de présentation visuelle/);assert.match(home.text,/App locale non connectée/);
assert.equal(home.response.headers.get('cache-control'),'private, no-store');
await check('/__dev/design',404);
await check('/__dev/components',404);
const icon=await check('/favicon.svg',200);assert.match(icon.text,/<svg/);
await check('/fonts/RobotoVariable.ttf',200);
const css=home.text.match(/href="([^"]+\.css)"/);assert.ok(css,'SSR asset link');await check(css[1],200);
await check('/api/v1/tools',503);
await check('/api/v1/tools',405,{method:'POST',body:'{}'});
await check('/api/v1/tools',403,{headers:{Origin:'https://untrusted.invalid'}});
await check('/api/v1/admin/publish',404);
writeFileSync(new URL('../../docs/evidence/workspace-ssr.json',import.meta.url),JSON.stringify({date:'2026-10-27',runtime:'built Worker in local workerd',apiConnected:false,checks},null,2)+'\n');
console.log(`Compiled SSR: ${checks.length} HTTP checks passed; no demonstration route or data`);
