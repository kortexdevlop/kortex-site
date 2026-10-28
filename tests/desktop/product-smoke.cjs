const {chromium,expect}=require('playwright/test');const fs=require('fs');
(async()=>{let browser;for(let i=0;i<120;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9342');break}catch{await new Promise(r=>setTimeout(r,250));}}if(!browser)throw Error('Application Windows indisponible');
try{const page=browser.contexts()[0].pages()[0];await expect(page.getByRole('navigation',{name:'Navigation principale'})).toBeVisible({timeout:30000});
const offline=process.env.KORTEX_DESKTOP_OFFLINE==='1';const proof={date:new Date().toISOString(),mode:offline?'offline':'online',passed:false};
if(!offline){
 const origin='https://kortex-api-staging.userpgl.workers.dev';const data=await(await fetch(origin+'/api/v1/tools?limit=24&sort=recent')).json();
 await expect(page.locator('.tool-card')).toHaveCount(data.items.length,{timeout:30000});for(const tool of data.items)await expect(page.getByRole('heading',{name:tool.name,exact:true})).toBeVisible();
 await page.getByLabel('Rechercher un outil',{exact:true}).fill('casse');await page.getByRole('button',{name:'Rechercher',exact:true}).click();await expect(page.locator('.tool-card')).toHaveCount(1);await expect(page.locator('.tool-card')).toContainText('Convertisseur de casse');
 await page.locator('.tool-card').click();await expect(page.getByRole('dialog',{name:'Fiche de l’outil'})).toContainText('Aucune permission sensible.');await expect(page.getByRole('dialog',{name:'Fiche de l’outil'})).toContainText('initiales');await page.getByRole('button',{name:'Fermer la fiche'}).click();
 await page.getByLabel('Rechercher un outil',{exact:true}).fill('aucun-outil-xyz-404');await page.getByRole('button',{name:'Rechercher',exact:true}).click();await expect(page.locator('.tool-card')).toHaveCount(0);await expect(page.getByText('Aucun outil ne correspond à votre recherche.',{exact:true})).toBeVisible();
 await page.getByLabel('Rechercher un outil',{exact:true}).fill('');await page.getByRole('button',{name:'Rechercher',exact:true}).click();await expect(page.locator('.tool-card')).toHaveCount(data.items.length);
 const category=data.items[0].category;const filtered=await(await fetch(origin+'/api/v1/tools?limit=24&sort=recent&category='+encodeURIComponent(category))).json();await page.getByLabel('Catégorie',{exact:true}).selectOption(category);await expect.poll(()=>page.locator('.tool-card h2').allTextContents()).toEqual(filtered.items.map(x=>x.name));await page.getByLabel('Catégorie',{exact:true}).selectOption('');await expect(page.locator('.tool-card')).toHaveCount(data.items.length);
 await page.getByLabel('Trier',{exact:true}).selectOption('name');const sorted=await(await fetch(origin+'/api/v1/tools?limit=24&sort=name')).json();await expect.poll(()=>page.locator('.tool-card h2').allTextContents()).toEqual(sorted.items.map(x=>x.name));
 proof.tools=data.items.map(x=>({id:x.id,sha256:x.sha256}));proof.real_catalogue=true;proof.search=true;proof.filters=true;proof.sort=true;proof.details_and_actions=true;
 await page.screenshot({path:'docs/evidence/desktop-store-windows.png',fullPage:true});
 for(const label of ['Bibliothèque','Mises à jour','Compte']){await page.getByRole('button',{name:label,exact:true}).click();await expect(page.getByRole('heading',{level:1})).toBeVisible();}
}else{
 await expect(page.getByText('Hors connexion · vos outils installés restent disponibles.',{exact:true})).toBeVisible();
 await page.getByRole('button',{name:'Bibliothèque',exact:true}).click();await expect(page.locator('.tool-card')).toHaveCount(1);
 await page.getByRole('button',{name:'Ouvrir',exact:true}).click();
 await expect(page.getByLabel('Texte à convertir',{exact:true})).toBeVisible({timeout:60000});await page.getByLabel('Texte à convertir',{exact:true}).fill('ÉTÉ ß');await page.getByRole('button',{name:'Majuscules',exact:true}).click();await expect(page.getByLabel('Texte converti',{exact:true})).toHaveValue('ÉTÉ SS');
 await expect(page.getByLabel('Paquet .tool signé',{exact:true})).toHaveCount(0);await expect(page.getByLabel('Paramètres JSON',{exact:true})).toHaveCount(0);await expect(page.getByRole('dialog')).toHaveCount(0);proof.signed_installed_tool_opened_without_network=true;proof.conversion='ÉTÉ SS';
 await page.screenshot({path:'docs/evidence/desktop-offline-windows.png',fullPage:true});
 await page.getByRole('button',{name:'Retour à la bibliothèque',exact:true}).click();await expect(page.getByRole('heading',{name:'Votre bibliothèque',exact:true})).toBeVisible();
}
proof.passed=true;fs.writeFileSync('docs/evidence/desktop-'+proof.mode+'-windows.json',JSON.stringify(proof,null,2)+'\n');console.log('PASS Windows product '+proof.mode);
}finally{await browser.close();}})().catch(e=>{console.error(e);process.exitCode=1});
