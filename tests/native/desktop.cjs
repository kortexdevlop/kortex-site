// Run against the Windows WebView2 instance launched with debugging port 9332.
const {chromium,expect}=require('playwright/test');
const fs=require('node:fs');
const path=require('node:path');
const root=path.resolve(__dirname,'../..');
(async()=>{
 const browser=await chromium.connectOverCDP('http://127.0.0.1:9332');
 try{
  const page=browser.contexts()[0].pages()[0];
  await expect(page.getByRole('heading',{name:'Kortex Desktop — expérimental'})).toBeVisible();
  await page.getByRole('button',{name:'Ouvrir',exact:true}).click();
  await expect(page.getByRole('dialog',{name:'Permissions de l’outil'})).toBeVisible();
  await page.getByRole('button',{name:'Annuler',exact:true}).click();
  await expect(page.locator('[data-node=result]')).toHaveCount(0);
  await page.getByRole('button',{name:'Ouvrir',exact:true}).click();
  await page.getByRole('button',{name:'Continuer sans accès sensibles'}).click();
  await expect(page.locator('[data-node=result]')).toHaveText('Ready',{timeout:60000});
  await page.getByLabel('Paramètres JSON').fill('{"a":25,"b":48}');
  await page.getByRole('button',{name:'Exécuter l’action'}).click();
  await expect(page.getByRole('status')).toContainText('1200');
  await page.screenshot({path:path.join(root,'docs/evidence/native-desktop-6.3.png'),fullPage:true});
  await page.getByLabel('Paramètres JSON').fill('invalid');
  await page.getByRole('button',{name:'Exécuter l’action'}).click();
  await expect(page.getByRole('status')).toContainText('JSON invalide');
  await expect(page.locator('[data-node=result]')).toHaveText('Ready');
  await page.getByRole('button',{name:'Fermer',exact:true}).click();
  await expect(page.locator('[data-node=result]')).toHaveCount(0);
  await expect(page.getByRole('status')).toHaveText('Fermé');
  const dir=path.join(root,'target/desktop-fixture');
  const bad=fs.readFileSync(path.join(dir,'demo.tool'));bad[40]^=1;
  fs.writeFileSync(path.join(dir,'tampered.tool'),bad);
  fs.copyFileSync(path.join(dir,'demo.tool.json'),path.join(dir,'tampered.tool.json'));
  await page.getByLabel('Paquet .tool signé').fill(path.join(dir,'tampered.tool'));
  await page.getByRole('button',{name:'Ouvrir',exact:true}).click();
  await page.getByRole('button',{name:'Continuer sans accès sensibles'}).click();
  await expect(page.getByRole('status')).toContainText('Paquet refusé');
  await expect(page.getByRole('heading',{name:'Kortex Desktop — expérimental'})).toBeVisible();
  console.log('PASS Windows Desktop: signed tool, shared Ready UI, multiply=1200, consent/cancel, close, invalid JSON and tampered archive.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
