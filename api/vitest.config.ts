import {cloudflareTest,readD1Migrations} from '@cloudflare/vitest-plugin';
import {defineConfig} from 'vitest/config';
import {readFileSync} from 'node:fs';
const fixture=(path:string)=>JSON.parse(readFileSync(new URL('../'+path,import.meta.url),'utf8'));
export default defineConfig(async()=>({plugins:[cloudflareTest({wrangler:{configPath:'./wrangler.jsonc'},miniflare:{bindings:{
 CURSOR_SIGNING_KEY:'93'.repeat(32), // Public TEST key only; never part of Wrangler deployment configuration.
 TEST_MIGRATIONS:await readD1Migrations('../database/migrations'),
 TEST_MANIFEST:fixture('tools/calculatrice/manifest.json'),TEST_CONTRACT:fixture('tools/calculatrice/contract.json'),TEST_DESCRIPTOR:fixture('tests/fixtures/artifact-signed-test.json')
}}})],test:{include:['tests/**/*.test.ts'],testTimeout:120000,hookTimeout:120000}}));
