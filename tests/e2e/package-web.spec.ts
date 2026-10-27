import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {serve,root} from '../web/harness';
const report=JSON.parse(readFileSync(resolve(root,'target/staging/calculatrice/release-report.json'),'utf8'));
const proof=JSON.parse(readFileSync(report.descriptor,'utf8'));
const bundle=JSON.parse(readFileSync(report.host_trust,'utf8'));
const fixture={descriptor:proof,bundle};
let server:Awaited<ReturnType<typeof serve>>;
test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>server.close());
test('paquet canonique du pipeline : signature racine, calculatrice UI/actions et reprise',async({page},info)=>{
 await page.route(server.origin+'/fixtures/release.tool',route=>route.fulfill({contentType:'application/zip',body:readFileSync(report.archive)}));
 await page.goto(server.origin);
 const open=()=>page.evaluate(async f=>{const api=await import('/adapter/interop.js' as string);await (window as any).kortexHost.loadTool({...f.descriptor,url:location.origin+'/fixtures/release.tool'},api.TrustStore.fromSignedKeys(f.bundle.document,f.bundle.root,f.bundle.minimum_version),{api_origin:location.origin,allowed_parent_origin:location.origin,tool_id:'calculatrice',tool_version:'1.0.0'})},fixture);
 await open();await expect(page.locator('[data-node=result]')).toHaveText('Prêt à calculer');
 const network:string[]=[];page.on('request',r=>network.push(r.url()));
 for(const key of ['digit-2','digit-5','target-b','digit-4','digit-8','multiply']){
  if(key==='multiply'){await page.locator(`[data-node="${key}"] button`).focus();await page.keyboard.press('Enter');}else{await page.locator(`[data-node="${key}"] button`).click();}
  // The guest owns state; wait for each acknowledgement before the next press.
  if(key==='digit-2')await expect(page.getByLabel('Premier nombre',{exact:true})).toHaveValue('2');
  if(key==='digit-5')await expect(page.getByLabel('Premier nombre',{exact:true})).toHaveValue('25');
  if(key==='target-b')await expect(page.locator('[data-node=target]')).toContainText('deuxième');
  if(key==='digit-4')await expect(page.getByLabel('Deuxième nombre',{exact:true})).toHaveValue('4');
  if(key==='digit-8')await expect(page.getByLabel('Deuxième nombre',{exact:true})).toHaveValue('48');
 }
 await expect(page.locator('[data-node=result]')).toHaveText('Résultat : 1200');
 const box=await page.locator('[data-node=digit-2] button').boundingBox();expect(box!.width).toBeGreaterThanOrEqual(44);expect(box!.height).toBeGreaterThanOrEqual(44);
 await page.evaluate(async()=>{await (window as any).kortexHost.command({kind:'action.invoke',payload:{action_id:'multiply',input:{a:25,b:48}}})});
 await expect(page.getByLabel('Résultat',{exact:true})).toContainText('1200');
 await page.getByLabel('Deuxième nombre',{exact:true}).fill('0');await expect(page.locator('[data-node=result]')).toHaveText('Prêt à calculer');
 await page.getByRole('button',{name:'Diviser',exact:true}).click();await expect(page.locator('[data-node=result]')).toHaveText('DIVIDE_BY_ZERO');
 await page.evaluate(async()=>{await (window as any).kortexHost.command({kind:'action.invoke',payload:{action_id:'divide',input:{a:25,b:0}}})});
 await expect(page.getByLabel('Résultat',{exact:true})).toContainText('DIVIDE_BY_ZERO');
 expect(network).toEqual([]);
 await page.screenshot({path:resolve(root,`docs/evidence/package8-${info.project.name}.png`),fullPage:true});
 await page.getByRole('button',{name:'Fermer l’outil',exact:true}).click();await expect(page.locator('[data-node=calculator]')).toHaveCount(0);
 await open();await expect(page.getByLabel('Premier nombre',{exact:true})).toHaveValue('');await expect(page.locator('[data-node=result]')).toHaveText('Prêt à calculer');
});
