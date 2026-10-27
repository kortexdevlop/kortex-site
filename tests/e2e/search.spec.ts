import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
test('recherche API, clavier, langue, texte et absence de résultat',async({page})=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto('/');await expect(page.locator('[data-search-dialog]')).toHaveAttribute('data-ready','true');await page.keyboard.press('Control+k');const dialog=page.locator('[data-search-dialog]');await expect(dialog).toBeVisible();const input=page.locator('#ws-live-query');await expect(input).toBeFocused();
 await input.fill('Calculatrice');await expect(dialog.locator('[data-search-result]')).toHaveCount(1);await input.press('ArrowDown');await expect(dialog.locator('[data-search-result]')).toBeFocused();await input.focus();await input.fill('aucun-resultat');await expect(dialog.getByRole('status')).toHaveText('Aucun résultat');
 await dialog.getByRole('combobox').selectOption('en');await expect(page.locator('html')).toHaveAttribute('lang','en');await expect(page.locator('#workspace-title')).toContainText('What would');await dialog.getByRole('button',{name:'Aa — Larger text'}).click();await expect(page.locator('.workspace')).toHaveAttribute('data-text-size','large');
 await page.keyboard.press('Escape');await expect(dialog).toBeHidden();await page.keyboard.press('Control+k');await expect(dialog).toBeVisible();await input.fill('Calculatrice');await expect(dialog.locator('[data-search-result]')).toHaveCount(1);
 await page.addScriptTag({content:readFileSync('node_modules/axe-core/axe.min.js','utf8')});const critical=await page.evaluate(async()=>{const result=await (window as any).axe.run(document.querySelector('[data-search-dialog]'));return result.violations.filter((v:any)=>v.impact==='critical').map((v:any)=>v.id);});expect(critical).toEqual([]);expect(errors).toEqual([]);
 await dialog.locator('[data-search-result]').click();await expect(page).toHaveURL(/\/tools\/calculatrice$/);await expect(page.locator('html')).toHaveAttribute('lang','en');
});
test('réponses obsolètes annulées, délai 250 ms et panne visible',async({page})=>{
 let requests=0;await page.route('**/api/v1/tools?**',async route=>{requests++;await route.fulfill({status:503,contentType:'application/json',body:'{}'});});await page.goto('/');await expect(page.locator('[data-search-dialog]')).toHaveAttribute('data-ready','true');await page.keyboard.press('Control+k');const input=page.locator('#ws-live-query');await input.pressSequentially('Calculatrice',{delay:10});await expect(page.locator('[data-search-dialog] [role=status]')).toContainText('indisponible');expect(requests).toBe(1);
});
test('une ancienne réponse ne remplace pas la recherche courante',async({page})=>{
 let settled:()=>void;const first=new Promise<void>(resolve=>settled=resolve);
 await page.route('**/api/v1/tools?**',async route=>{if(new URL(route.request().url()).searchParams.get('q')==='Photo'){try{const response=await route.fetch();await new Promise(resolve=>setTimeout(resolve,600));await route.fulfill({response});}catch{}finally{settled();}}else await route.continue();});
 await page.goto('/');await expect(page.locator('[data-search-dialog]')).toHaveAttribute('data-ready','true');await page.keyboard.press('Control+k');const input=page.locator('#ws-live-query');await input.fill('Photo');await expect(page.locator('[data-search-dialog] [role=status]')).toContainText('cours');await input.fill('Calculatrice');await expect(page.locator('[data-search-result]')).toHaveCount(1);await first;await expect(page.locator('[data-search-result]')).toHaveText('Calculatrice');
});
