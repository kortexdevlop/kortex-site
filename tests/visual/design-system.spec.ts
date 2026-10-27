import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
const captureDirectory=process.env.KORTEX_DESIGN_CAPTURE_DIR||'docs/evidence';
const axe=readFileSync('node_modules/axe-core/axe.min.js','utf8');
for(const [name,width,height] of [['desktop',1632,927],['tablet',768,1024],['mobile',390,844]] as const){
  test(`design library ${name}: reflow, focus, axe and reproducible viewport`,async({page},info)=>{
    await page.setViewportSize({width,height});await page.emulateMedia({colorScheme:'light',reducedMotion:'reduce'});await page.goto('/__dev/components');
    await page.evaluate(()=>document.fonts.ready);
    await expect(page.locator('#demo-cards .tool-card')).toHaveCount(9);
    expect(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth)).toBe(true);
    await page.addScriptTag({content:axe});
    const violations=await page.evaluate(async()=> (await (window as any).axe.run(document,{runOnly:{type:'tag',values:['wcag2a','wcag2aa','wcag21aa']}})).violations.map((v:any)=>({id:v.id,targets:v.nodes.map((n:any)=>n.target)})));
    expect(violations).toEqual([]);
    await page.screenshot({path:`${captureDirectory}/library-${info.project.name}-${name}.png`,animations:'disabled'});
    await page.locator('#demo-cards').screenshot({path:`${captureDirectory}/library-${info.project.name}-${name}-cards.png`,animations:'disabled'});
    const first=page.locator('#demo-cards a').first();await first.focus();await page.keyboard.press('Tab');await expect(page.locator('#demo-cards a').nth(1)).toBeFocused();
  });
}
test('SSR has no fabricated catalog and returns 503 when its service is missing',async({page})=>{
  const response=await page.goto('/');expect(response?.status()).toBe(503);
  await expect(page.getByRole('alert')).toContainText('indisponible');
  await expect(page.locator('.ws-tool-card')).toHaveCount(0);
  const rejected=await page.request.post('/api/v1/admin/publish',{data:{}});expect(rejected.status()).toBe(404);
  const unavailable=await page.request.get('/api/v1/tools');expect(unavailable.status()).toBe(503);
});
