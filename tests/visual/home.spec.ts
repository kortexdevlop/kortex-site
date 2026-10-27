import {test,expect} from 'playwright/test';
test('accueil de référence, cinq vrais outils et recherche API',async({page},info)=>{
 await page.goto('/?q=Photo');await expect(page.locator('h1')).toHaveText("Que voulez-vous faire aujourd'hui ?");await expect(page.locator('.ws-tool-card')).toHaveCount(5);await expect(page.locator('.ws-recent-card')).toHaveCount(0);await expect(page.locator('.ws-connection')).toContainText('non connectée');
 expect(await page.locator('.ws-tool-grid').evaluate(el=>getComputedStyle(el).gridTemplateColumns.split(' ').length)).toBe(4);
 await page.locator('#ws-task').fill('Calculatrice');await page.getByRole('button',{name:'Trouver le bon outil'}).click();await expect(page).toHaveURL(/\/tools\?q=Calculatrice/);await expect(page.locator('.ws-tool-card')).toHaveCount(1);
 await page.locator('.ws-tool-open').click();await expect(page).toHaveURL(/\/tools\/calculatrice$/);
 for(const width of [2706,1353,768,390,320]){await page.setViewportSize({width,height:width===2706?1764:882});await page.goto('/?q=Photo');expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);await expect(page.locator('.ws-composer')).toBeVisible();if(width===1353)await page.screenshot({path:`docs/evidence/home-real-${info.project.name}.png`});}
});
