import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';

export function parseEnv(text) {
  const result = {};
  for (const line of text.split(/\r?\n/)) {
    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith('#')) continue;
    const match = trimmed.match(/^([A-Z][A-Z0-9_]*)=(.*)$/);
    if (!match) throw new Error('Invalid environment assignment');
    if (Object.hasOwn(result, match[1])) throw new Error(`Duplicate variable: ${match[1]}`);
    result[match[1]] = match[2];
  }
  return result;
}
const origins = ['PUBLIC_WEB_ORIGIN', 'PUBLIC_API_ORIGIN', 'PUBLIC_RUN_ORIGIN'];
const remote = ['OIDC_ISSUER','OIDC_CLIENT_ID','OIDC_REDIRECT_URI','CLOUDFLARE_ACCOUNT_ID','CLOUDFLARE_D1_DATABASE_ID','CLOUDFLARE_R2_BUCKET','KORTEX_SIGNING_KEY_ID','KORTEX_SIGNING_PUBLIC_KEY'];
export function validateEnv(env, { example = false } = {}) {
  const errors = [];
  const mode = env.KORTEX_ENV;
  if (!['local','staging','production'].includes(mode)) errors.push('environment: KORTEX_ENV must be local, staging or production');
  for (const key of Object.keys(env)) {
    if (key.startsWith('PUBLIC_') && !origins.includes(key)) errors.push(`public-secret: unexpected public variable ${key}`);
    if (['KORTEX_SIGNING_PRIVATE_KEY','KORTEX_SIGNING_KEY_SEED_BASE64'].includes(key)) errors.push('ci-only: private signing key forbidden in application configuration');
  }
  for (const key of origins) {
    if (!env[key]) { errors.push(`missing: ${key}`); continue; }
    try {
      const url = new URL(env[key]);
      if (!['http:','https:'].includes(url.protocol) || url.username || url.password || url.search || url.hash || url.pathname !== '/') throw Error();
      if (mode !== 'local' && url.protocol !== 'https:') throw Error();
      if (mode !== 'local' && ['localhost','127.0.0.1','[::1]'].includes(url.hostname)) throw Error();
    } catch { errors.push(`origin: invalid ${key}`); }
  }
  if (env.PUBLIC_WEB_ORIGIN === env.PUBLIC_RUN_ORIGIN) errors.push('isolation: runtime must have a distinct origin');
  if (!example && mode !== 'local') {
    for (const key of remote) if (!env[key]?.trim()) errors.push(`missing: ${key}`);
  }
  for (const key of ['OIDC_ISSUER','OIDC_REDIRECT_URI']) if (env[key]) {
    try { const url = new URL(env[key]); if (url.protocol !== 'https:' || url.username || url.password) throw Error(); }
    catch { errors.push(`identity: invalid ${key}`); }
  }
  return errors;
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const example = process.argv.includes('--example');
    const env = example ? parseEnv(readFileSync('.env.example','utf8')) : process.env;
    const errors = validateEnv(env, { example });
    if (errors.length) { console.error(errors.join('\n')); process.exitCode = 1; }
    else console.log(`Configuration ${env.KORTEX_ENV} validated${example ? ' (non-secret example)' : ''}`);
  } catch (error) { console.error(error.message); process.exitCode = 1; }
}
