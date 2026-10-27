import {chromium} from 'playwright';
import {writeFileSync} from 'node:fs';
const browser=await chromium.launch({ignoreDefaultArgs:['--hide-scrollbars']});
try {
  const page=await browser.newPage({viewport:{width:2706,height:1764},deviceScaleFactor:1,reducedMotion:'reduce'});
  const errors=[];page.on('pageerror',error=>errors.push(error.message));
  const response=await page.goto('http://127.0.0.1:4321/__dev/design');
  if(response.status()!==200)throw Error(`Preview HTTP ${response.status()}`);
  await page.evaluate(()=>document.fonts.ready);await page.mouse.move(0,0);
  const boxes={};for(const selector of ['.ws-sidebar','.ws-main-panel','.ws-topbar','#workspace-title','.ws-subtitle','.ws-composer','.ws-suggestions','.ws-recent','.ws-recent-card','.ws-library','.ws-tool-card','.ws-connection','.ws-account']) boxes[selector]=await page.locator(selector).first().boundingBox();
  await page.screenshot({path:'test-results/workspace-current.png'});
  writeFileSync('docs/evidence/workspace-geometry.json',JSON.stringify({viewport:{width:2706,height:1764},boxes,errors},null,2)+'\n');
  console.log(JSON.stringify({boxes,errors}));
  if(errors.length)throw Error('Workspace browser errors');
}finally{await browser.close();}
