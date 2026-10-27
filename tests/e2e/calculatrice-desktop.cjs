// Test-only CDP connection, enabled only by the companion harness.
const {chromium,expect}=require('playwright/test');
const fs=require('node:fs'),path=require('node:path');const root=path.resolve(__dirname,'../..');
(async()=>{const browser=await chromium.connectOverCDP('http://127.0.0.1:9332');try{
 const page=browser.contexts()[0].pages()[0];
 await expect(page.getByRole('button',{name:'Ouvrir',exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Ouvrir',exact:true}).click();await page.getByRole('button',{name:'Annuler',exact:true}).click();await expect(page.locator('[data-node=calculator]')).toHaveCount(0);
 const open=async()=>{await page.getByRole('button',{name:'Ouvrir',exact:true}).click();await page.getByRole('button',{name:'Continuer sans accès sensibles'}).click();await expect(page.locator('[data-node=result]')).toHaveText('Prêt à calculer',{timeout:60000})};
 await open();
 // Dioxus transports local WebView events through a virtual host, not the Internet.
 const network=[];let ipc=0;page.on('request',r=>{const u=new URL(r.url());if(u.origin==='http://dioxus.index.html'&&u.pathname==='//__events'){ipc++;}else{network.push(r.url());}});
 for(const [key,field,value] of [['digit-2','Premier nombre','2'],['digit-5','Premier nombre','25'],['target-b',null,null],['digit-4','Deuxième nombre','4'],['digit-8','Deuxième nombre','48'],['multiply',null,null]]){
  if(key==='multiply'){await page.locator(`[data-node="${key}"] button`).focus();await page.keyboard.press('Enter');}else{await page.locator(`[data-node="${key}"] button`).click();}if(field)await expect(page.getByLabel(field,{exact:true})).toHaveValue(value);if(key==='target-b')await expect(page.locator('[data-node=target]')).toContainText('deuxième');
 }
 await expect(page.locator('[data-node=result]')).toHaveText('Résultat : 1200');
 const box=await page.locator('[data-node=digit-2] button').boundingBox();expect(box.width).toBeGreaterThanOrEqual(44);expect(box.height).toBeGreaterThanOrEqual(44);
 await page.getByLabel('Action',{exact:true}).fill('multiply');await page.getByLabel('Paramètres JSON').fill('{"a":25,"b":48}');await page.getByRole('button',{name:'Exécuter l’action'}).click();await expect(page.getByLabel('Résultat',{exact:true})).toContainText('1200');
 await page.getByLabel('Deuxième nombre',{exact:true}).fill('0');await expect(page.locator('[data-node=result]')).toHaveText('Prêt à calculer');
 await page.getByRole('button',{name:'Diviser',exact:true}).click();await expect(page.locator('[data-node=result]')).toHaveText('DIVIDE_BY_ZERO');
 await page.getByLabel('Action',{exact:true}).fill('divide');await page.getByLabel('Paramètres JSON').fill('{"a":25,"b":0}');await page.getByRole('button',{name:'Exécuter l’action'}).click();await expect(page.getByLabel('Résultat',{exact:true})).toContainText('DIVIDE_BY_ZERO');
 expect(network).toEqual([]);expect(ipc).toBeGreaterThan(0);
 await page.screenshot({path:path.join(root,'docs/evidence/calculatrice-windows.png'),fullPage:true});
 await page.getByRole('button',{name:'Fermer',exact:true}).click();await expect(page.locator('[data-node=calculator]')).toHaveCount(0);await open();await expect(page.getByLabel('Premier nombre',{exact:true})).toHaveValue('');
 const f=JSON.parse(fs.readFileSync(path.join(root,'target/calculatrice-fixture/fixture.json')));console.log('PASS Windows calculatrice: keypad/action=1200, identical DIVIDE_BY_ZERO, cancel/close/reopen; archive SHA256 '+f.descriptor.sha256);
}finally{await browser.close()}})().catch(e=>{console.error(e);process.exitCode=1});
