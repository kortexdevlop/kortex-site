// Compare precompiled CSP-safe validators with the independent golden corpus.
// Generate the corpus first: node --test tests/contracts/cross-language.test.mjs
import {createRequire} from 'node:module';
import {readFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import assert from 'node:assert/strict';
const require=createRequire(new URL('../../web-adapter/package.json',import.meta.url));
const {build}=require('esbuild');
const output=resolve('target/contracts/csp-validators.mjs');
mkdirSync(resolve('target/contracts'),{recursive:true});
await build({entryPoints:['web-adapter/src/browser-contracts.ts'],outfile:output,bundle:true,platform:'node',format:'esm'});
const validators=await import(pathToFileURL(output).href);
const cases=JSON.parse(readFileSync('target/contracts/cases.json','utf8')).filter(c=>c.kind==='manifest'||c.kind==='contract');
for(const c of cases){const validate=c.kind==='manifest'?validators.validateManifest:validators.validateContract;assert.equal(validate(c.value).ok,c.expected,c.name);}
console.log(`PASS ${cases.length} frozen manifest/contract cases with CSP-safe validators.`);
