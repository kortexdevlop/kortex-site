import {test,expect} from 'playwright/test';
test('depuis la fiche : iframe signée, permissions et calcul local 1200',async({page},info)=>{
 const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto('/tools/calculatrice');await page.getByRole('link',{name:'Ouvrir',exact:true}).click();await expect(page).toHaveURL(/\/app\/calculatrice$/);
 await expect(page.locator('meta[name=robots]')).toHaveAttribute('content','noindex, nofollow');await expect(page.locator('iframe')).toHaveAttribute('sandbox','allow-scripts allow-same-origin');
 await page.evaluate(()=>window.dispatchEvent(new MessageEvent('message',{origin:'https://evil.invalid',source:document.querySelector('iframe')!.contentWindow,data:{kind:'RESULT',nonce:'a'.repeat(64),session_id:'00000000-0000-4000-8000-000000000000',result:{tool_id:'calculatrice',version:'1.0.0',status:'ready'}}})));
 await expect(page.locator('[data-runtime-viewer]')).toHaveAttribute('data-state','ready',{timeout:30000});
 const frame=page.frameLocator('iframe');await expect(frame.locator('[data-node=result]')).toHaveText('Prêt à calculer');
 await frame.getByLabel('Premier nombre',{exact:true}).fill('25');await expect(frame.getByLabel('Premier nombre',{exact:true})).toHaveValue('25');
 await frame.getByLabel('Deuxième nombre',{exact:true}).fill('48');await expect(frame.getByLabel('Deuxième nombre',{exact:true})).toHaveValue('48');await frame.getByRole('button',{name:'Multiplier',exact:true}).click();await expect(frame.locator('[data-node=result]')).toHaveText('Résultat : 1200');
 expect(await page.evaluate(()=>{try{void document.querySelector('iframe')!.contentWindow!.localStorage;return false;}catch{return true;}})).toBe(true);
 expect(errors).toEqual([]);await frame.locator('[data-node=result]').scrollIntoViewIfNeeded();await page.screenshot({path:`docs/evidence/open-tool-${info.project.name}.png`});
 const response=await page.reload();expect(response!.headers()['content-security-policy']).toContain('frame-src http://127.0.0.1:4340');expect(response!.headers()['cache-control']).toBe('private, no-store');
});
test('archive altérée : erreur explicite et aucune exécution',async({page})=>{
 await page.route('http://127.0.0.1:4340/calculatrice.tool',route=>route.fulfill({status:200,contentType:'application/zip',body:'tampered archive'}));
 await page.goto('/app/calculatrice');await expect(page.locator('[data-runtime-viewer]')).toHaveAttribute('data-state','error',{timeout:30000});
 await expect(page.locator('[data-runtime-status]')).toContainText('Impossible d’ouvrir');await expect(page.frameLocator('iframe').locator('[data-node=calculator]')).toHaveCount(0);
});
