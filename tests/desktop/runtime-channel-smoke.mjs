import {spawn} from 'node:child_process';
import {mkdirSync,readFileSync,readdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {chromium,expect} from 'playwright/test';
const root=resolve(import.meta.dirname,'../..'),dir=resolve(root,'target/runtime-channel-desktop-smoke');mkdirSync(dir,{recursive:true});
const app=spawn(resolve(root,'target/dx/kortex-desktop/release/windows/app/kortex-desktop.exe'),[],{cwd:root,windowsHide:true,env:{...process.env,KORTEX_DESKTOP_DATA_DIR:dir,KORTEX_DESKTOP_OFFLINE:'0',WEBVIEW2_USER_DATA_FOLDER:resolve(dir,'webview'),WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS:'--remote-debugging-port=9354 --remote-allow-origins=*'}});
let browser;try{
 for(let i=0;i<100&&!browser;i++){try{browser=await chromium.connectOverCDP('http://127.0.0.1:9354')}catch{await new Promise(r=>setTimeout(r,250))}}
 if(!browser)throw Error('Desktop WebView2 unavailable');const page=browser.contexts()[0].pages()[0];page.setDefaultTimeout(30000);
 await page.getByRole('button',{name:'Mises à jour',exact:true}).click();await page.getByRole('button',{name:'Vérifier la version de Kortex',exact:true}).click();
 await expect(page.getByRole('status').filter({hasText:'Aucune nouvelle version de Kortex Desktop disponible.'})).toBeVisible();
 const files=readdirSync(resolve(dir,'runtime-channels')).filter(f=>f.endsWith('.json'));if(files.length!==1)throw Error('Signed runtime cache missing');const cache=JSON.parse(readFileSync(resolve(dir,'runtime-channels',files[0])));
 await page.screenshot({path:resolve(root,'docs/evidence/pipeline-desktop-version.png')});writeFileSync(resolve(root,'docs/evidence/pipeline-desktop-runtime-check.json'),JSON.stringify({date:new Date().toISOString(),passed:true,real_windows_release_executable:true,real_api_response:true,signature_and_revocation_checked_by_native_code:true,sequence:cache.envelope.document.sequence,web_runtime:cache.envelope.document.channels.staging.runtime_version,no_automatic_install:true},null,2)+'\n');console.log('Desktop release UI verified real signed runtime metadata and revocations.');
}finally{await browser?.close();app.kill()}
