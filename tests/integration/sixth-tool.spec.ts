import {test,expect} from 'playwright/test';
test('sixième outil publié sans modification des clients : vrai paquet et action echo',async({page})=>{
 await page.goto('https://kortex-web-staging.userpgl.workers.dev/tools/extension-echo-check');await page.getByRole('link',{name:'Ouvrir',exact:true}).click();
 await expect(page.locator('[data-runtime-viewer]')).toHaveAttribute('data-state','ready',{timeout:45000});
 const frame=page.frames().find(f=>f.url().startsWith('https://kortex-run-staging.userpgl.workers.dev'))!;
 await expect(frame.locator('[data-node=echo]')).toContainText('Extension temporaire prête');
 const out=await frame.evaluate(async()=>{await (window as any).kortexHost.command({kind:'action.invoke',payload:{action_id:'echo',input:{text:'Extension autonome ✓'}}});return (window as any).kortexHost.snapshot().result;});
 expect(out).toEqual({text:'Extension autonome ✓'});await expect(frame.getByRole('button',{name:'Refuser',exact:true})).toHaveCount(0);
 await page.screenshot({path:'docs/evidence/extension-staging.png',fullPage:true});
});
