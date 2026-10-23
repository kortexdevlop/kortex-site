import {test,expect} from 'playwright/test';
import {serve} from './harness';
let server:Awaited<ReturnType<typeof serve>>;
test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>{await server.close()});
test('shell réel sans outil et modes visuels',async({page})=>{
 await page.goto(server.origin);await expect(page.getByRole('heading',{name:'Kortex',exact:true})).toBeVisible();
 await expect(page.locator('main')).toHaveAttribute('data-status','unavailable');
 const select=page.getByRole('combobox',{name:'Mode d’affichage'});await select.selectOption('contrast');await expect(page.locator('main')).toHaveAttribute('data-mode','contrast');
 await select.focus();await page.keyboard.press('Tab');await expect(page.getByRole('button',{name:'Fermer l’outil'})).toBeFocused();
 await page.context().setOffline(true);await expect(page.locator('main')).toHaveAttribute('data-status','offline');await page.context().setOffline(false);
});
test('loading et error visibles sans prétendre charger un outil',async({page})=>{
 await page.addInitScript(()=>{(window as any).kortexHost={snapshot:()=>({status:'loading'}),command:async()=>{}}});
 await page.goto(server.origin);await expect(page.locator('main')).toHaveAttribute('data-status','loading');
 await page.evaluate(()=>{(window as any).kortexHost.snapshot=()=>({status:'error'});window.dispatchEvent(new Event('kortex:host'));});
 await expect(page.getByRole('alert')).toContainText('L’outil n’a pas pu être exécuté');
});
