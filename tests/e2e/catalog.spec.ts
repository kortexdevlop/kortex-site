import {test,expect} from 'playwright/test';
import {executeSQL,toolSQL} from '../website/fixtures.mjs';
test('catalogue SSR paginé, filtres et ajout D1 sans reconstruction',async({page},info)=>{
 const response=await page.goto('/tools');expect(response!.status()).toBe(200);expect(response!.headers()['cache-control']).toBe('public, max-age=60');
 await expect(page.locator('.ws-tool-card')).toHaveCount(24);await expect(page.getByRole('link',{name:'Page suivante'})).toBeVisible();
 const ids=await page.locator('.ws-tool-card').evaluateAll(nodes=>nodes.map(n=>(n as HTMLElement).dataset.id));
 await page.getByRole('link',{name:'Page suivante'}).click();const next=await page.locator('.ws-tool-card').evaluateAll(nodes=>nodes.map(n=>(n as HTMLElement).dataset.id));expect(next.length).toBeGreaterThan(0);expect(next.some(id=>ids.includes(id))).toBe(false);
 await page.goto('/tools?category=image&sort=name');await expect(page.locator('.ws-tool-card')).toHaveCount(5);await expect(page.locator('.ws-tool-open').first()).toContainText('Photo 0');
 await page.goto('/tools?q=aucun-resultat');await expect(page.locator('.ws-tool-card')).toHaveCount(0);
 const id='ajout-'+info.project.name;executeSQL(toolSQL(id,'Ajout '+info.project.name,{time:100}));
 await page.goto('/tools/'+id);await expect(page.locator('.tool-details h1')).toHaveText('Ajout '+info.project.name);
 await page.goto('/tools?q='+id.replace('ajout-','Ajout '));await expect(page.locator('.ws-tool-card')).toHaveCount(1);
});
test('fiche réelle, métadonnées échappées, cachés 404 et indisponibilité 503',async({page,request},info)=>{
 const response=await page.goto('/tools/calculatrice');expect(response!.status()).toBe(200);await expect(page.locator('.ws-detail h1')).toHaveText('Calculatrice');await expect(page.getByRole('link',{name:'Ouvrir',exact:true})).toHaveAttribute('href','/app/calculatrice');
 await expect(page.locator('link[rel=canonical]')).toHaveAttribute('href','http://127.0.0.1:4321/tools/calculatrice');
 for(const id of ['outil-cache','inexistant']){const r=await page.goto('/tools/'+id);expect(r!.status()).toBe(404);await expect(page.locator('meta[name=robots]')).toHaveAttribute('content','noindex, nofollow');await expect(page.locator('link[rel=canonical]')).toHaveCount(0);}
 const id='escaped-'+info.project.name;executeSQL(toolSQL(id,'<script>alert(1)</script>'));
 await page.goto('/tools/'+id);await expect(page.locator('.ws-detail h1')).toHaveText('<script>alert(1)</script>');expect(await page.locator('h1 script').count()).toBe(0);
 await expect.poll(async()=>(await request.get('http://127.0.0.1:4331/tools')).status()).toBe(503);
 await page.goto('http://127.0.0.1:4331/tools');await expect(page.getByRole('alert')).toContainText('indisponible');
 expect((await request.get('http://127.0.0.1:4331/tools/calculatrice')).status()).toBe(503);
});
