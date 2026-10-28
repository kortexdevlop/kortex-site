import assert from 'node:assert/strict';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve} from 'node:path';
import {spawnSync} from 'node:child_process';
import {randomBytes,randomUUID} from 'node:crypto';
import {chromium,expect} from 'playwright/test';
const root=resolve(import.meta.dirname,'..'),dir=resolve(root,'target/password-staging');mkdirSync(dir,{recursive:true});
const json=p=>JSON.parse(readFileSync(resolve(root,p),'utf8'));
const owner=json('target/cloud-staging-resources.json'),apiConfig=json('api/wrangler.jsonc'),siteConfig=json('website/wrangler.jsonc');
assert.equal(owner.worker,apiConfig.env.staging.name);assert.equal(owner.database_id,apiConfig.env.staging.d1_databases[0].database_id);assert.equal(owner.bucket,apiConfig.env.staging.r2_buckets[0].bucket_name);assert.equal(siteConfig.env.staging.name,'kortex-web-staging');
const api='https://kortex-api-staging.userpgl.workers.dev',web='https://kortex-web-staging.userpgl.workers.dev';
const proof={date:new Date().toISOString(),staging_only:true,production_touched:false,dns_touched:false,mail_touched:false,google_enabled:false,created_secrets:[],passed:false};
const save=()=>writeFileSync(resolve(root,'docs/evidence/password-auth-staging.json'),JSON.stringify(proof,null,2)+'\n');
function cli(args){const result=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,...(args[0]==='whoami'?[]:['--profile','kortex'])],{cwd:root,encoding:'utf8',windowsHide:true,timeout:180000,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'}});if(result.status!==0)throw Error('Cloudflare '+args[0]+' failed: '+result.stderr.slice(-500));return result.stdout}
assert.ok(JSON.parse(cli(['whoami','--json'])).accounts.some(a=>a.id===owner.account));const cloudToken=JSON.parse(cli(['auth','token','--json'])).token;
async function cloud(path,init={}){const response=await fetch('https://api.cloudflare.com/client/v4/accounts/'+owner.account+path,{...init,headers:{Authorization:'Bearer '+cloudToken,'Content-Type':'application/json'},signal:AbortSignal.timeout(30000)});const data=await response.json();assert.ok(response.ok&&data.success,'Cloudflare request failed');return data.result}
async function query(sql,params=[]){return cloud('/d1/database/'+owner.database_id+'/query',{method:'POST',body:JSON.stringify({sql,params})})}
let browser;const fixtureEmail='password-check-'+randomUUID().slice(0,8)+'@example.invalid',fixturePassword='Kortex test phrase '+randomBytes(18).toString('base64url');
try{
 const secrets=await cloud('/workers/scripts/'+owner.worker+'/secrets');
 for(const name of ['SESSION_HMAC_SECRET','PASSWORD_PEPPER'])if(!secrets.some(s=>s.name===name)){await cloud('/workers/scripts/'+owner.worker+'/secrets',{method:'PUT',body:JSON.stringify({name,type:'secret_text',text:randomBytes(48).toString('base64url')})});proof.created_secrets.push(name)}
 const migrations=cli(['d1','migrations','apply','kortex-catalog-staging','--config','api/wrangler.jsonc','--env','staging','--remote']);writeFileSync(resolve(dir,'migrations.log'),migrations);proof.migrations_applied=true;
 writeFileSync(resolve(dir,'api-deploy.log'),cli(['deploy','--config','api/wrangler.jsonc','--env','staging']));proof.api_deployed=true;
 const compiled=resolve(root,'website/dist/server'),generated=json('website/dist/server/wrangler.json');assert.equal(generated.kv_namespaces.length,0);
 const deployment={...generated,name:'kortex-web-staging',topLevelName:'kortex-web-staging',configPath:resolve(dir,'web.json'),userConfigPath:resolve(dir,'web.json'),main:resolve(compiled,generated.main),assets:{...generated.assets,directory:resolve(compiled,generated.assets.directory)},services:siteConfig.env.staging.services,vars:siteConfig.env.staging.vars,workers_dev:true,preview_urls:false};
 assert.ok(deployment.services.some(s=>s.binding==='AUTH_WORKER'&&s.entrypoint==='AuthService'&&s.service===owner.worker));writeFileSync(resolve(dir,'web.json'),JSON.stringify(deployment));writeFileSync(resolve(dir,'web-deploy.log'),cli(['deploy','--config',resolve(dir,'web.json')]));proof.web_deployed=true;
 const options=await(await fetch(web+'/api/v1/auth/options')).json();assert.equal(options.password,true);assert.equal(options.google,false);proof.email_independent_of_google=true;
 assert.equal((await fetch(api+'/api/v1/auth/password/token')).status,404);proof.direct_api_cookie_routes_blocked=true;
 browser=await chromium.launch({headless:true});const context=await browser.newContext({viewport:{width:1360,height:950}}),page=await context.newPage();
 await page.goto(web+'/auth/register');await expect(page.getByRole('button',{name:'Créer mon compte',exact:true})).toBeEnabled({timeout:30000});await page.screenshot({path:resolve(root,'docs/evidence/password-auth-register-desktop.png'),fullPage:true});
 await page.setViewportSize({width:390,height:844});assert.ok(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth));await page.screenshot({path:resolve(root,'docs/evidence/password-auth-register-mobile.png'),fullPage:true});
 await page.getByLabel('Adresse email',{exact:true}).fill(fixtureEmail);await page.getByLabel('Mot de passe',{exact:true}).fill(fixturePassword);await page.getByLabel('Confirmer le mot de passe',{exact:true}).fill(fixturePassword);await page.getByRole('button',{name:'Créer mon compte',exact:true}).click();await page.waitForURL('**/account',{timeout:45000});await expect(page.getByText(fixtureEmail,{exact:true})).toBeVisible();await expect(page.getByText('Adresse email non vérifiée',{exact:true})).toBeVisible();proof.registration_and_account=true;
 const sessionCookie=(await context.cookies()).find(c=>c.name==='__Host-kortex_session');assert.ok(sessionCookie?.secure&&sessionCookie.httpOnly&&sessionCookie.sameSite==='Lax'&&sessionCookie.path==='/'&&sessionCookie.domain===new URL(web).hostname);proof.secure_cookie=true;
 await page.reload();await expect(page.getByText(fixtureEmail,{exact:true})).toBeVisible();proof.session_survives_reload=true;
 const forged=await page.evaluate(async()=> (await fetch('/api/v1/auth/logout',{method:'POST',credentials:'same-origin'})).status);assert.equal(forged,403);proof.logout_requires_csrf=true;
 await page.getByRole('button',{name:'Se déconnecter',exact:true}).click();await page.waitForURL('**/auth/login');await expect(page.getByRole('button',{name:'Se connecter',exact:true})).toBeEnabled();assert.ok(!(await context.cookies()).some(c=>c.name==='__Host-kortex_session'));proof.logout_revokes_session=true;
 await page.getByLabel('Adresse email',{exact:true}).fill(fixtureEmail);await page.getByLabel('Mot de passe',{exact:true}).fill('Un mot de passe incorrect 2026');await page.getByRole('button',{name:'Se connecter',exact:true}).click();await expect(page.getByRole('status')).toHaveText('Email ou mot de passe incorrect.',{timeout:45000});proof.wrong_password_rejected=true;
 await page.getByLabel('Mot de passe',{exact:true}).fill(fixturePassword);await page.getByRole('button',{name:'Se connecter',exact:true}).click();await page.waitForURL('**/account',{timeout:45000});await expect(page.getByText(fixtureEmail,{exact:true})).toBeVisible();proof.password_login=true;
 await page.setViewportSize({width:1360,height:950});await page.screenshot({path:resolve(root,'docs/evidence/password-auth-account-desktop.png'),fullPage:true});
 proof.passed=true;console.log('PASS real Chromium email registration, login, session and logout on staging');
}catch(error){proof.error=String(error.message).replaceAll(fixturePassword,'[redacted]').replaceAll(fixtureEmail,'[test-account]').slice(0,1200);console.error('Email authentication validation failed; see sanitized evidence.');process.exitCode=1}
finally{
 if(browser)await browser.close();
 try{assert.match(fixtureEmail,/^password-check-[0-9a-f]{8}@example\.invalid$/);await query('DELETE FROM users WHERE id IN(SELECT user_id FROM password_credentials WHERE email=?)',[fixtureEmail]);const remaining=await query('SELECT count(*) AS n FROM password_credentials WHERE email=?',[fixtureEmail]);assert.equal(remaining[0].results[0].n,0);proof.test_account_removed=true}catch{proof.cleanup_failed=true;proof.passed=false;process.exitCode=1}
 save();
}
