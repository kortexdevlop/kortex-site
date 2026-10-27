import {test,expect} from 'playwright/test';
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const axe=readFileSync('node_modules/axe-core/axe.min.js','utf8');
test('new reference identity and native desktop geometry',async({page},info)=>{
  const source=readFileSync('docs/reference/workspace-selection.png');
  expect(createHash('sha256').update(source).digest('hex')).toBe('22b72dd91b43675643c8bf996074690916bec86b5ebd55d785aeff2fae603067');
  expect([source.readUInt32BE(16),source.readUInt32BE(20)]).toEqual([2706,1764]);
  await page.setViewportSize({width:2706,height:1764});await page.goto('/__dev/design');await page.evaluate(()=>document.fonts.ready);await page.mouse.move(0,0);
  for(const [selector,expected] of [['.ws-sidebar',[26,26,444,1712]],['.ws-main-panel',[498,26,2182,1712]],['.ws-topbar',[498,26,2182,126]],['.ws-composer',[815,458,1520,280]],['.ws-recent-card',[578,1008,479,132]],['.ws-tool-card',[578,1320,479,304]],['.ws-connection',[50,1414,396,176]]] as const){
    const box=await page.locator(selector).first().boundingBox();expect(box).not.toBeNull();for(const [index,key] of ['x','y','width','height'].entries())expect(Math.abs(box![key as keyof typeof box]-expected[index]),`${selector} ${key}`).toBeLessThanOrEqual(4);
  }
  await expect(page.locator('.ws-tool-card')).toHaveCount(14);await expect(page.locator('.ws-recent-card')).toHaveCount(3);
  await expect(page.locator('.ws-tool-grid')).toHaveCSS('grid-template-columns',/.* /);
  expect(await page.locator('.ws-tool-grid').evaluate(el=>getComputedStyle(el).gridTemplateColumns.split(' ').length)).toBe(4);
  await page.screenshot({path:`docs/evidence/workspace-${info.project.name}-reference.png`});
  await page.addScriptTag({content:axe});const results=await page.evaluate(async()=>await (window as any).axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}}));
  writeFileSync(`docs/evidence/workspace-${info.project.name}-accessibility.json`,JSON.stringify({violations:results.violations.map((v:any)=>({id:v.id,impact:v.impact,nodes:v.nodes.map((n:any)=>({target:n.target,summary:n.failureSummary}))}))},null,2)+'\n');
  expect(results.violations.filter((v:any)=>v.id!=='color-contrast').map((v:any)=>({id:v.id,targets:v.nodes.map((n:any)=>n.target)}))).toEqual([]);
});
test('search, categories, modes, favorites and list are functional',async({page})=>{
  await page.setViewportSize({width:1353,height:882});await page.goto('/__dev/design');
  await page.locator('#ws-global-query').fill('PDF');await expect(page.locator('.ws-tool-card:visible')).toHaveCount(2);
  await page.locator('[data-new-task]').click();await expect(page.locator('#ws-task')).toBeFocused();
  await page.locator('[data-category=image]').click();await expect(page.locator('.ws-tool-card:visible')).toHaveCount(3);
  await page.locator('[data-new-task]').click();await page.getByRole('radio',{name:'Local',exact:true}).focus();await page.keyboard.press('Space');await expect(page.getByRole('radio',{name:'Local',exact:true})).toBeChecked();await expect(page.locator('.ws-tool-card:visible')).toHaveCount(6);
  await page.locator('[data-new-task]').click();await page.locator('[data-section=favorites]').click();await expect(page.locator('.ws-tool-card:visible')).toHaveCount(2);
  await page.locator('.ws-tool-card:visible .ws-favorite').first().focus();await page.keyboard.press('Enter');await expect(page.locator('.ws-tool-card:visible')).toHaveCount(1);
  await page.reload();await expect(page.locator('[data-count=favorites]')).toHaveText('1');
  await page.locator('[data-view=list]').click();await expect(page.locator('.ws-tool-grid')).toHaveAttribute('data-layout','list');
  await page.locator('[data-view=grid]').click();await page.locator('[data-suggestion="PDF vers Word"]').click();await expect(page.locator('.ws-tool-card:visible')).toHaveCount(2);
});
test('file remains local, dialogs and shortcuts work',async({page})=>{
  await page.goto('/__dev/design');
  let posts=0;page.on('request',request=>{if(request.method()==='POST')posts++;});
  await page.locator('#ws-file').setInputFiles({name:'mes-photos.zip',mimeType:'application/zip',buffer:Buffer.from('local test')});
  await expect(page.locator('.ws-file-status')).toContainText('mes-photos.zip');expect(posts).toBe(0);
  await page.getByRole('button',{name:'Docs',exact:true}).click();await expect(page.getByRole('dialog')).toBeVisible();await page.keyboard.press('Escape');await expect(page.getByRole('dialog')).toBeHidden();
  await page.keyboard.press('Control+k');await expect(page.locator('#ws-global-query')).toBeFocused();
  await page.keyboard.press('Control+n');await expect(page.locator('#ws-task')).toBeFocused();await expect(page.locator('.ws-file-status')).toBeHidden();
});
for(const [width,height] of [[768,1024],[390,844],[320,740]])test(`workspace reflows and mobile navigation works at ${width}`,async({page},info)=>{
  await page.setViewportSize({width,height});await page.goto('/__dev/design');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  const menu=page.getByRole('button',{name:'Ouvrir le menu'});await menu.click();await expect(page.locator('.ws-sidebar')).toBeVisible();await expect(menu).toHaveAttribute('aria-expanded','true');
  await page.keyboard.press('Escape');await expect(menu).toBeFocused();await expect(page.locator('.ws-sidebar')).toBeHidden();
  await page.screenshot({path:`docs/evidence/workspace-${info.project.name}-${width}.png`});
  await menu.click();await page.locator('[data-category=documents]').click();await expect(page.locator('.ws-sidebar')).toBeHidden();await expect(page.locator('.ws-tool-card:visible')).toHaveCount(2);
});
test('public workspace never substitutes the screenshot tools or a false desktop connection',async({page})=>{
  const response=await page.goto('/');expect(response?.status()).toBe(503);
  await expect(page.locator('.ws-tool-card')).toHaveCount(0);await expect(page.locator('.ws-recent-card')).toHaveCount(0);
  await expect(page.locator('.ws-connection')).toContainText('App locale non connectée');await expect(page.getByRole('alert')).toContainText('indisponible');
  await expect(page.locator('body')).not.toContainText('photos-vacances.zip');
});
