import {test, expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
const axe = readFileSync('node_modules/axe-core/axe.min.js','utf8');

test('desktop navigation, keyboard focus, preferences and persistent state', async ({page}, info) => {
  await page.setViewportSize({width:1632,height:927});
  await page.emulateMedia({colorScheme:'light'});
  await page.goto('/__dev/design');
  await expect(page.getByRole('heading',{level:1})).toBeVisible();
  await page.keyboard.press('Tab');
  await expect(page.getByRole('link',{name:'Aller au contenu'})).toBeFocused();
  await page.keyboard.press('Enter');
  await expect(page.locator('#main')).toBeFocused();
  await expect(page.locator('nav a')).toHaveCount(4);
  await expect(page.locator('#header-panel')).toBeVisible();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.goto('/__dev/design');
  await page.screenshot({path:`docs/evidence/navigation-${info.project.name}-desktop.png`,fullPage:true});
  await page.getByRole('button',{name:'Thème sombre',exact:true}).click();
  await page.getByRole('button',{name:'Agrandir le texte'}).click();
  await page.getByLabel('Densité',{exact:true}).selectOption('pro');
  await page.getByRole('button',{name:'Passer en anglais'}).click();
  await expect(page.locator('html')).toHaveAttribute('lang','en');
  await expect(page.locator('nav a').first()).toHaveText('Tools');
  await expect(page.locator('#global-search')).toHaveAttribute('placeholder','Search for a tool');
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme','dark');
  await expect(page.locator('html')).toHaveAttribute('data-type','large');
  await expect(page.locator('html')).toHaveAttribute('data-density','pro');
  await expect(page.locator('#theme-toggle')).toHaveAttribute('aria-pressed','true');
  await expect(page.locator('#language-toggle')).toHaveText('EN');
  await page.screenshot({path:`docs/evidence/navigation-${info.project.name}-dark.png`,fullPage:true});
});

test('mobile disclosure supports keyboard, Escape, resizing and 320px reflow', async ({page},info) => {
  await page.setViewportSize({width:390,height:844});
  await page.goto('/__dev/design');
  const menu=page.locator('#menu-toggle');
  await expect(menu).toBeVisible();
  await expect(page.locator('#header-panel')).toBeHidden();
  await menu.focus(); await page.keyboard.press('Enter');
  await expect(menu).toHaveAttribute('aria-expanded','true');
  await page.keyboard.press('Tab');
  await expect(page.locator('nav a').first()).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(menu).toBeFocused();
  await expect(page.locator('#header-panel')).toBeHidden();
  await menu.click();
  await page.getByRole('link',{name:'Applications',exact:true}).click();
  await expect(page.locator('#header-panel')).toBeHidden();
  await page.goto('/__dev/design');
  await page.screenshot({path:`docs/evidence/navigation-${info.project.name}-mobile.png`,fullPage:true});
  await page.setViewportSize({width:320,height:740});
  await menu.click();
  await page.locator('#type-toggle').click();
  expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
  await page.setViewportSize({width:1632,height:927});
  await expect(page.locator('#header-panel')).toBeVisible();
  await expect(menu).toBeHidden();
});

test('light and dark accessible names, contrast and markup; real local search submission', async ({page}) => {
  await page.setViewportSize({width:1632,height:927});
  await page.goto('/__dev/design');
  for (const theme of ['light','dark']) {
    await page.evaluate(theme=>{document.documentElement.dataset.theme=theme;},theme);
    await page.addScriptTag({content:axe});
    const result=await page.evaluate(async()=>await (window as any).axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}}));
    expect(result.violations.map((v:any)=>`${v.id}: ${v.nodes.map((n:any)=>n.target).join(',')}`)).toEqual([]);
  }
  await page.locator('#demo-search').fill('<test & query>');
  await page.locator('#demo-search').press('Enter');
  await expect(page.locator('#search-demo [role="status"]')).toContainText('<test & query>');
  await expect(page.locator('#search-demo script')).toHaveCount(0);
  await page.locator('#sample-action').click();
  await expect(page.locator('#action-result')).toHaveText('Le bouton répond.');
});

test('storage failure, system theme and no JavaScript leave a usable document', async ({browser}) => {
  const context=await browser.newContext({javaScriptEnabled:false,viewport:{width:390,height:844}});
  const page=await context.newPage(); await page.goto('http://127.0.0.1:4321/__dev/design');
  await expect(page.locator('nav a').first()).toBeVisible();
  await expect(page.locator('#global-search')).toBeVisible();
  await context.close();
  const blocked=await browser.newContext({colorScheme:'dark'});
  await blocked.addInitScript(()=>{Object.defineProperty(window,'localStorage',{get(){throw new Error('blocked');}});});
  const p=await blocked.newPage(); await p.goto('http://127.0.0.1:4321/__dev/design');
  await expect(p.locator('#theme-toggle')).toHaveAttribute('aria-pressed','true');
  await p.locator('#theme-toggle').click();
  await expect(p.locator('html')).toHaveAttribute('data-theme','light');
  await blocked.close();
});
