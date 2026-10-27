import {test,expect} from 'playwright/test';
test('cards use a 3/2/1 grid, escaped metadata, clamped titles and host-owned icons',async({page})=>{
  await page.goto('/__dev/components');
  const grid=page.locator('#demo-cards .tool-grid');
  await expect(grid.locator('.tool-card')).toHaveCount(9);
  for(const [width,height,columns] of [[1632,927,3],[768,1024,2],[390,844,1]]){
    await page.setViewportSize({width,height});
    expect(await grid.evaluate(el=>getComputedStyle(el).gridTemplateColumns.split(' ').length)).toBe(columns);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  }
  await expect(grid).toContainText('Texte <sans HTML> & détails');
  await expect(grid.locator('img,script,iframe')).toHaveCount(0);
  expect(await grid.locator('h3').last().evaluate(el=>getComputedStyle(el).webkitLineClamp)).toBe('2');
  const link=grid.locator('a').first();await link.focus();await expect(link).toBeFocused();
  await expect(link).toHaveAttribute('href','/tools/demo-visual-1');
});
test('loading/empty/error feedback and hydrated filtering work without demo publication',async({page})=>{
  await page.goto('/__dev/components');
  await expect(page.locator('#demo-states [aria-busy=true]')).toBeVisible();
  await expect(page.locator('#demo-states .skeleton')).toHaveCount(3);
  await expect(page.locator('#demo-states .feedback-empty')).toContainText('Aucun outil');
  await expect(page.locator('#demo-states [role=alert]')).toContainText('indisponible');
  await page.locator('astro-island[component-export="default"]').waitFor();
  await page.waitForFunction(()=>!document.querySelector('#demo-filter astro-island')?.hasAttribute('ssr'));
  await page.locator('#catalog-query').fill('Convertir');
  await page.locator('#catalog-query').press('Enter');
  await expect(page.locator('#demo-filter .tool-card')).toHaveCount(1);
  await page.locator('#catalog-query').fill('aucun-resultat-impossible');await page.locator('#catalog-query').press('Enter');
  await expect(page.locator('#demo-filter')).toContainText('Aucun outil');
});
