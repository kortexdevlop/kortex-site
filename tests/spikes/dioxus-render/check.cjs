const {chromium}=require('playwright');const fs=require('fs');const path=require('path');const http=require('http');
const root=path.resolve(__dirname,'../../..');
async function verify(page,label){

 const {expect}=require('playwright/test');
 await expect(page.getByRole('heading',{name:'Kortex renderer prototype'})).toBeVisible();
 await expect(page.getByRole('heading',{name:'Overview'})).toBeVisible();
 await expect(page.getByRole('table')).toBeVisible();await expect(page.getByRole('img',{name:'Values bar chart'})).toBeVisible();
 const input=page.getByRole('textbox',{name:'Search'});await input.fill('Kortex input');await expect(page.locator('#input-state')).toHaveText('Input: Kortex input');
 await input.focus();await expect(input).toBeFocused();
 const button=page.getByRole('button',{name:'Run',exact:true});const before=Number((await page.locator('#click-state').innerText()).split(':')[1]);await button.click();await expect(page.locator('#click-state')).toHaveText('Clicks: '+(before+1));
 await button.focus();await expect(button).toBeFocused();
 const snapshot=await page.locator('main').evaluate(main=>({text:main.innerText.replace(/Clicks: \d+/g,'Clicks: <count>'),keys:[...main.querySelectorAll('[data-key]')].map(n=>n.getAttribute('data-key')),bars:[...main.querySelectorAll('svg rect')].map(n=>[n.getAttribute('x'),n.getAttribute('y'),n.getAttribute('height')])}));
 const out=path.join(root,'docs/evidence');fs.mkdirSync(out,{recursive:true});await page.screenshot({path:path.join(out,`spike-3.2-${label}.png`),fullPage:true});return snapshot;
}
(async()=>{
 const desktop=await chromium.connectOverCDP('http://127.0.0.1:9332');const page=desktop.contexts()[0].pages()[0];const native=await verify(page,'desktop');console.log('Desktop WebView assertions PASS');
 if(process.argv.includes('--desktop-only')){await desktop.close();return;}
 const webRoot=process.env.KORTEX_WEB_DIST;if(!webRoot)throw Error('KORTEX_WEB_DIST required');
 const server=http.createServer((req,res)=>{try{let filename=path.resolve(webRoot,'.'+new URL(req.url,'http://local').pathname);if(filename===path.resolve(webRoot))filename=path.join(filename,'index.html');if(!filename.startsWith(path.resolve(webRoot)+path.sep))throw Error();const types={'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css'};res.setHeader('Content-Type',types[path.extname(filename)]||'application/octet-stream');res.end(fs.readFileSync(filename));}catch{res.statusCode=404;res.end()}});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));const browser=await chromium.launch({headless:true});
 try{const web=await browser.newPage({viewport:{width:1000,height:900}});await web.goto(`http://127.0.0.1:${server.address().port}`);const rendered=await verify(web,'chrome');require('assert/strict').deepEqual(rendered,native);console.log('Chrome assertions and Desktop/Web semantic parity PASS');}finally{await browser.close();await desktop.close();await new Promise(resolve=>server.close(resolve));}
})().catch(error=>{console.error(error);process.exitCode=1});
