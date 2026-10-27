const {chromium,expect}=require('playwright/test');
const fs=require('node:fs'),path=require('node:path');
const {makeTool}=require('../fixtures/make-tool.cjs');
const root=path.resolve(__dirname,'../..');
const dir=path.join(root,'target/native-cap-fixture');fs.mkdirSync(dir,{recursive:true});
function write(name,options){const f=makeTool(options),file=path.join(dir,name+'.tool');fs.writeFileSync(file,f.bytes);fs.writeFileSync(file+'.json',JSON.stringify(f.descriptor));fs.writeFileSync(path.join(dir,name+'-trust.json'),JSON.stringify(f.trust));return[file,path.join(dir,name+'-trust.json')];}
(async()=>{
 const browser=await chromium.connectOverCDP('http://127.0.0.1:9332');
 try{
  const page=browser.contexts()[0].pages()[0];
  const fixture=write('storage',{permissions:['storage.read'],wasmPath:path.join(root,'target/web-test-guest/wasm32-unknown-unknown/release/kortex_browser_capability_test.wasm')});
  async function open(files){await page.getByLabel('Paquet .tool signé').fill(files[0]);await page.getByLabel('Clés publiques hôte (JSON)').fill(files[1]);await page.getByRole('button',{name:'Ouvrir',exact:true}).click();await page.getByRole('button',{name:'Continuer sans accès sensibles'}).click();}
  for(const allow of [false,true]){
   await open(fixture);await expect(page.locator('[data-node=outcome]')).toHaveText('pending',{timeout:60000});
   await expect(page.getByRole('dialog',{name:'Demande d’autorisation'})).toContainText('storage.read');
   await page.getByRole('button',{name:allow?'Autoriser une fois':'Refuser',exact:true}).click();
   if(allow){await expect(page.locator('[data-node=outcome]')).toContainText('resolved:');await page.screenshot({path:path.join(root,'docs/evidence/native-capabilities.png'),fullPage:true});}
   else await expect(page.locator('[data-node=outcome]')).toHaveText('PERMISSION');
   await page.getByRole('button',{name:'Fermer',exact:true}).click();await expect(page.locator('[data-node=outcome]')).toHaveCount(0);
  }
  const bad=path.join(dir,'empty.wasm');fs.writeFileSync(bad,Buffer.from([0,97,115,109,1,0,0,0]));await open(write('unsupported',{wasmPath:bad}));await expect(page.getByRole('status')).toContainText('UnsupportedAbi');
  await open(write('recovery',{}));await expect(page.locator('[data-node=result]')).toHaveText('Ready',{timeout:60000});await page.getByRole('button',{name:'Fermer',exact:true}).click();
  console.log('PASS native signed SDK guest: refuse/allow, capability.result UI, close/reopen, ABI failure and recovery.');
 }finally{await browser.close();}
})().catch(e=>{console.error(e);process.exitCode=1;});
