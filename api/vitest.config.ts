import {cloudflareTest,readD1Migrations} from '@cloudflare/vitest-plugin';
import {defineConfig} from 'vitest/config';
import {readFileSync} from 'node:fs';
import {generateKeyPairSync} from 'node:crypto';
const fixture=(path:string)=>JSON.parse(readFileSync(new URL('../'+path,import.meta.url),'utf8'));
const testKeys=generateKeyPairSync('ed25519'),testPublic={...testKeys.publicKey.export({format:'jwk'}),kid:'ephemeral-api-test',alg:'EdDSA',use:'sig'};
const testPolicy={issuer:'urn:kortex:staging-operator',audience:'kortex-api-test',subject:'local-api-tests',repository:'test/kortex',repository_id:'1',ref:'refs/heads/test',workflow_ref:'test/kortex/.github/workflows/test.yml@refs/heads/test',actions:['publish','revoke','rollback'],jwks:{keys:[testPublic]}};
export default defineConfig(async()=>({plugins:[cloudflareTest({wrangler:{configPath:'./wrangler.jsonc'},miniflare:{bindings:{
 CURSOR_SIGNING_KEY:'93'.repeat(32), // Public TEST key only; never part of Wrangler deployment configuration.
 TEST_MIGRATIONS:await readD1Migrations('../database/migrations'),
 ADMIN_AUTH_POLICY:JSON.stringify(testPolicy),TEST_AUTH_PRIVATE_JWK:testKeys.privateKey.export({format:'jwk'}),
 TEST_ARCHIVE_BASE64:readFileSync(new URL('../tests/fixtures/artifact-signed-test.tool',import.meta.url)).toString('base64'),
 TEST_MANIFEST:fixture('tests/fixtures/calculatrice-v1-manifest.json'),TEST_CONTRACT:fixture('tools/calculatrice/contract.json'),TEST_DESCRIPTOR:fixture('tests/fixtures/artifact-signed-test.json')
}}})],test:{fileParallelism:false,include:['tests/**/*.test.ts'],testTimeout:120000,hookTimeout:120000}}));
