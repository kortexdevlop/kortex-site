import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
const proof=JSON.parse(readFileSync('docs/evidence/web-hot-publish.json','utf8'));
if(!proof.passed||!proof.operator_policy_restored)throw Error('A successful real staging hot-publication gate is required; no mock or skip permitted.');
test('preuve Cloudflare : déploiement inchangé, exécution signée et révocation effective',async({page,request})=>{
 expect(proof.site_build_before).toBe(proof.site_build_after);expect(proof.site_deployment_before).toBe(proof.site_deployment_after);expect(proof.browsers).toEqual([{name:'chromium',result:1200},{name:'webkit',result:1200}]);
 expect((await request.get(proof.web_origin+'/tools/'+proof.tool_id)).status()).toBe(404);expect((await request.get(proof.web_origin+'/app/'+proof.tool_id)).status()).toBe(404);
 expect((await request.get(proof.run_origin+'/api/v1/downloads/'+proof.tool_id+'/1.0.0')).status()).toBe(410);expect((await request.get(proof.api_origin+'/api/v1/internal/sitemap?page=0')).status()).toBe(404);
 const response=await page.goto(proof.web_origin);expect(response!.status()).toBe(200);expect(response!.headers()['x-robots-tag']).toBe('noindex, nofollow');
 await expect(page.locator('[data-search-dialog]')).toHaveAttribute('data-ready','true',{timeout:15000});await page.keyboard.press('Control+k');await page.locator('#ws-live-query').fill('Calculatrice');await expect(page.locator('[data-search-result]')).toHaveCount(1,{timeout:15000});
});
