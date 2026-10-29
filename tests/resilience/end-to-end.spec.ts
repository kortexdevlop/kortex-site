import {test,expect} from 'playwright/test';
import {desktop,open,action} from '../e2e/workspace-helpers';
import {readFileSync,mkdirSync,writeFileSync} from 'node:fs';
const vectors=JSON.parse(readFileSync('tests/integration/five-tools-vectors.json','utf8'));
const ids=['calculatrice','convertisseur-casse','calculateur-hash','timestamp-utc','explorateur-csv'];
test('five real signed tools keep their actions offline and persist no inputs or results',async({page,context})=>{
 await desktop(page);const windows=[];for(const id of ids)windows.push({id,...await open(page,id)});
 await context.setOffline(true);await expect(page.locator('.desk-offline')).toBeVisible();await expect(page.getByRole('button',{name:'Ouvrir une fenêtre',exact:true})).toBeDisabled();
 for(const w of windows){const v=vectors.find((x:any)=>x.tool===w.id);await action(page,w.instance,v.action,v.input);await expect.poll(async()=>{try{return JSON.parse(await page.locator('.desk-actions pre').innerText())}catch{return null}}).toEqual(v.output)}
 const storage=await page.evaluate(()=>({layout:JSON.parse(localStorage.getItem('kortex.workspace.v1')!),history:JSON.parse(localStorage.getItem('kortex.workspace.history.v1')!)}));expect(storage.history).toHaveLength(5);expect(storage.history.every((r:any)=>Object.keys(r).sort().join(',')==='action_id,at,status,tool_id')).toBe(true);expect(storage.layout.windows.every((w:any)=>!('outputs'in w)&&!('input'in w))).toBe(true);await context.setOffline(false);
});
test('catalogue API failure is visible and loaded guest recovers without data loss',async({page})=>{
 await desktop(page);const w=await open(page,'calculatrice');await action(page,w.instance,'multiply',{a:25,b:48});await expect(page.locator('.desk-actions pre')).toContainText('1200');
 await page.route('**/api/v1/tools?*',route=>route.fulfill({status:503,json:{error:{code:'UNAVAILABLE',message:'QA isolated outage'}}}));
 await page.getByRole('button',{name:'Rechercher',exact:true}).click();await expect(page.locator('.desk-notice').first()).toContainText('Catalogue indisponible');await expect(w.window).toHaveAttribute('data-runtime-state','ready');await action(page,w.instance,'add',{a:2,b:3});await expect(page.locator('.desk-actions pre')).toContainText('5');
 await page.unroute('**/api/v1/tools?*');await page.getByRole('button',{name:'Rechercher',exact:true}).click();await expect(page.getByRole('button',{name:'Rechercher',exact:true})).toBeEnabled();await expect(page.locator('.desk-window')).toHaveCount(1);
});
test('an unavailable download fails visibly, then a fresh real launch succeeds',async({page})=>{
 await desktop(page);await page.route('**/api/v1/downloads/**',route=>route.fulfill({status:503,body:'QA unavailable'}));await page.getByLabel('Outil à ouvrir',{exact:true}).selectOption('calculatrice');await page.getByRole('button',{name:'Ouvrir une fenêtre',exact:true}).click();await expect(page.locator('.desk-window')).toHaveCount(1);await expect(page.locator('.desk-window[data-runtime-state=error]')).toHaveCount(1,{timeout:60000});
 await page.locator('.desk-window').getByRole('button',{name:/^Fermer /}).click();await page.unroute('**/api/v1/downloads/**');const w=await open(page,'calculatrice');await action(page,w.instance,'multiply',{a:25,b:48});await expect(page.locator('.desk-actions pre')).toContainText('1200');
});
test('AI network outage and expired proposal never start a guest or erase manual access',async({page})=>{
 let frames=0;page.on('request',r=>{if(new URL(r.url()).pathname==='/frame.html')frames++});await page.goto('/ai');await page.getByRole('button',{name:'Essayer un exemple local',exact:true}).click();await page.clock.install();await page.clock.fastForward(300001);await page.getByRole('button',{name:'Approuver l’action 1',exact:true}).click();await expect(page.locator('[data-ai-state="0"]')).toContainText('Échec');expect(frames).toBe(0);
 let accountFailures=0;await page.route('**/api/v1/me',r=>{accountFailures++;return r.abort()});await page.route('**/api/v1/ai/plan',r=>r.abort());await page.getByLabel('Votre demande',{exact:true}).fill('Un calcul');await page.getByLabel(/J’autorise l’envoi/).check();await page.getByRole('button',{name:'Demander une proposition',exact:true}).click();await expect(page.getByRole('status')).toContainText('indisponible');await expect(page.getByRole('link',{name:'Ouvrir les outils sans assistant',exact:true})).toHaveAttribute('href','/tools');expect(frames).toBe(0);expect(accountFailures).toBeGreaterThan(0);
});
test.afterAll(()=>{mkdirSync('target/ci-reports/resilience',{recursive:true});writeFileSync('target/ci-reports/resilience/scope.json',JSON.stringify({date:new Date().toISOString(),faults:'test-only browser interception; local real API/D1/R2 and signed guests; no cloud mutation',production_fault_injection:false},null,2)+'\n')});
