import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { parseEnv, validateEnv } from '../../scripts/check-env.mjs';
const sample = parseEnv(readFileSync('.env.example','utf8'));

test('local template is valid and isolates runtime', () => {
  assert.deepEqual(validateEnv(sample, {example:true}), []);
  assert.ok(validateEnv({...sample, PUBLIC_RUN_ORIGIN:sample.PUBLIC_WEB_ORIGIN}).some(e=>e.startsWith('isolation:')));
});
test('remote environments require identities and HTTPS', () => {
  for (const mode of ['staging','production']) {
    const errors=validateEnv({...sample,KORTEX_ENV:mode});
    assert.ok(errors.includes('missing: OIDC_ISSUER'));
    assert.ok(errors.includes('origin: invalid PUBLIC_WEB_ORIGIN'));
  }
});
test('public secret names and CI private keys are rejected without printing values', () => {
  const errors=validateEnv({...sample,PUBLIC_API_TOKEN:'simulated-secret',KORTEX_SIGNING_PRIVATE_KEY:'simulated-private',KORTEX_SIGNING_KEY_SEED_BASE64:'simulated-seed'});
  assert.ok(errors.some(e=>e.startsWith('public-secret:')));
  assert.ok(errors.some(e=>e.startsWith('ci-only:')));
  assert.ok(!errors.join().includes('simulated'));
});
test('missing origins, credentials in URLs and duplicate assignments are rejected', () => {
  assert.ok(validateEnv({...sample,PUBLIC_WEB_ORIGIN:''}).includes('missing: PUBLIC_WEB_ORIGIN'));
  assert.ok(validateEnv({...sample,PUBLIC_WEB_ORIGIN:'https://user:pass@example.com'}).some(e=>e.startsWith('origin:')));
  assert.throws(()=>parseEnv('A=1\nA=2'));
});
export function containsSecret(text) {
  return /-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----/.test(text)
    || /\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{30,}|AKIA[A-Z0-9]{16})\b/.test(text)
    || /^PUBLIC_[A-Z0-9_]*(?:SECRET|TOKEN|PASSWORD|PRIVATE_KEY)\s*=/m.test(text);
}
test('simulated credentials are detected', () => {
  assert.ok(containsSecret('ghp_'+'A'.repeat(36)));
  assert.ok(containsSecret('-----BEGIN '+'PRIVATE KEY-----'));
  assert.ok(containsSecret('PUBLIC_ACCESS_TOKEN=example'));
});
test('tracked files have no recognizable private keys or forbidden public assignments', () => {
  const files=execFileSync('git',['ls-files','-z'],{encoding:'utf8'}).split('\0').filter(Boolean);
  for (const file of files) assert.equal(containsSecret(readFileSync(file,'utf8')),false,`Forbidden secret pattern in ${file}`);
});
