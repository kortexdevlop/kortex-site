import {test,expect,type Page,type Frame} from 'playwright/test';
import {readFileSync,mkdtempSync,rmSync} from 'node:fs';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
const proof=JSON.parse(readFileSync('docs/evidence/phase12-staging.json','utf8'));if(!proof.passed||!proof.operator_policy_restored||!proof.tools.some((t:{id:string;published:boolean})=>t.id==='explorateur-csv'&&t.published))throw Error('Real staging publication of explorateur-csv required; no mocks/skips.');
async function open(page:Page){const response=await page.goto(proof.web_origin+'/tools/explorateur-csv');expect(response!.status()).toBe(200);await page.getByRole('link',{name:'Ouvrir',exact:true}).click();await expect(page.locator('[data-runtime-viewer]')).toHaveAttribute('data-state','ready',{timeout:45000});return page.frames().find(f=>f.url().startsWith(proof.run_origin))!;}
async function action(frame:Frame,id:string,input:unknown){await frame.evaluate(async({id,input})=>{await (window as any).kortexHost.command({kind:'action.invoke',payload:{action_id:id,input}});},{id,input});return frame.evaluate(()=>(window as any).kortexHost.snapshot().result);}
test('Cloudflare : explorateur CSV signé, fichier, stats, graphique, export, stockage et refus',async({playwright},info)=>{
 const profile=mkdtempSync(join(tmpdir(),'kortex-csv-closure-'));
 const context=await playwright.chromium.launchPersistentContext(profile,{channel:'chrome',headless:true,acceptDownloads:true});const page=await context.newPage();try{
 const lines=['id,"nom, complet",valeur'];for(let i=0;i<120;i++)lines.push(`${i},"ligne ${i} ""q""",${i*2}`);const buffer=Buffer.from(lines.join('\r\n')+'\r\n','utf8'),text=buffer.toString('utf8');
 let f=await open(page);const traffic:string[]=[];page.on('request',r=>traffic.push(r.url()));
 await f.getByRole('button',{name:'Choisir un fichier CSV',exact:true}).click();await f.getByRole('button',{name:'Refuser',exact:true}).click();await expect(f.locator('[data-node=status]')).toContainText('PERMISSION');await expect(f.locator('[data-node=tabs]')).toHaveCount(0);
 f=await open(page);
 await f.getByRole('button',{name:'Choisir un fichier CSV',exact:true}).click();const chooser=page.waitForEvent('filechooser');await f.getByRole('button',{name:'Autoriser une fois',exact:true}).click();await(await chooser).setFiles({name:'données.csv',mimeType:'text/csv',buffer});await f.getByRole('button',{name:'Autoriser pour la session',exact:true}).click();
 await expect(f.locator('[data-node=range]')).toContainText('120 ligne(s) sur 120 · page 1 / 3',{timeout:30000});await expect(f.locator('[data-node=rows] tbody tr')).toHaveCount(50);
 await f.getByRole('tab',{name:'Statistiques',exact:true}).click();await expect(f.locator('[data-node=stats_table] tbody tr')).toHaveCount(3);await expect(f.locator('[data-node=stats_table] tbody tr').nth(2)).toContainText('238');
 await f.getByRole('tab',{name:'Graphique',exact:true}).click();await expect(f.locator('[data-node=chart] svg rect.kortex-series')).toHaveCount(120);
 const result=await action(f,'summarize_csv',{data_text:text});expect(result.rows).toBe(120);expect(result.stats[2]).toEqual({name:'valeur',kind:'number',count:120,nulls:0,min:0,max:238,mean:119});
 // Real browser permission, real IndexedDB, real sandboxed download. No API mocking.
 await f.getByRole('button',{name:'Exporter le rapport',exact:true}).click();
 const downloadPromise=page.waitForEvent('download');
 await f.getByRole('button',{name:'Autoriser une fois',exact:true}).click();
 const download=await downloadPromise;expect(download.suggestedFilename()).toBe('rapport-csv.csv');
 await download.saveAs('docs/evidence/phase12-csv-export.csv');
 const saved=await download.path();expect(saved).toBeTruthy();
 expect(readFileSync(saved!,'utf8')).toBe((await action(f,'export_report',{data_text:text})).report_csv);
 await f.getByRole('button',{name:'Exporter le rapport',exact:true}).click();
 const forbiddenDownload=page.waitForEvent('download',{timeout:1500}).then(()=>true,()=>false);
 await f.getByRole('button',{name:'Refuser',exact:true}).click();
 await expect(f.locator('[data-node=notice]')).toContainText('PERMISSION');
 expect(await forbiddenDownload).toBe(false);
 const cdp=await page.context().newCDPSession(page);
 await cdp.send('Browser.setPermission',{permission:{name:'persistent-storage'},setting:'granted'});
 await f.getByRole('button',{name:'Mémoriser ce fichier',exact:true}).click();
 await f.getByRole('button',{name:'Autoriser pour la session',exact:true}).click();
 await expect(f.locator('[data-node=notice]')).toContainText('Nom du fichier mémorisé');
 expect(await f.evaluate(()=>navigator.storage.persisted())).toBe(true);
 f=await open(page);
 await f.getByRole('button',{name:'Afficher les fichiers récents',exact:true}).click();
 await f.getByRole('button',{name:'Refuser',exact:true}).click();
 await expect(f.locator('[data-node=notice]')).toContainText('PERMISSION');
 await expect(f.locator('[data-node=recent]')).not.toContainText('données.csv');
 f=await open(page);
 await f.getByRole('button',{name:'Afficher les fichiers récents',exact:true}).click();
 await f.getByRole('button',{name:'Autoriser pour la session',exact:true}).click();
 await expect(f.locator('[data-node=notice]')).toContainText('1 fichier(s) récent(s) chargé(s)');
 await expect(f.locator('[data-node=recent]')).toContainText('données.csv');
 await page.screenshot({path:`docs/evidence/phase12-staging-csv-${info.project.name}.png`,fullPage:true});
 await cdp.send('Browser.resetPermissions');
 await cdp.detach();
 expect(traffic.filter(url=>!url.startsWith(proof.web_origin)&&!url.startsWith(proof.run_origin)&&!/^https:\/\/fonts\.(googleapis|gstatic)\.com\//.test(url))).toEqual([]);
 f=await open(page);await f.getByRole('button',{name:'Choisir un fichier CSV',exact:true}).click();const bad=page.waitForEvent('filechooser');await f.getByRole('button',{name:'Autoriser une fois',exact:true}).click();await(await bad).setFiles({name:'mauvais.csv',mimeType:'text/csv',buffer:Buffer.from('a,b\n1,"2\n')});await f.getByRole('button',{name:'Autoriser pour la session',exact:true}).click();await expect(f.locator('[data-node=status]')).toContainText('UNTERMINATED_QUOTE');await expect(f.locator('[data-node=tabs]')).toHaveCount(0);
 }finally{await context.close();if(!profile.startsWith(join(tmpdir(),'kortex-csv-closure-')))throw Error('Unexpected profile path');rmSync(profile,{recursive:true,force:true});}
});
test('Cloudflare : cinq paquets publiés et neuf vecteurs identiques au bureau',async({page})=>{
 const vectors=JSON.parse(readFileSync('tests/integration/five-tools-vectors.json','utf8'));
 for(const id of [...new Set(vectors.map((v:any)=>v.tool))]){
  await page.goto(proof.web_origin+'/tools/'+id);
  await page.getByRole('link',{name:'Ouvrir',exact:true}).click();
  await expect(page.locator('[data-runtime-viewer]')).toHaveAttribute('data-state','ready',{timeout:45000});
  await expect(page.locator('iframe')).toHaveAttribute('sandbox','allow-scripts allow-same-origin allow-downloads');
  const frame=page.frames().find(f=>f.url().startsWith(proof.run_origin))!;
  for(const vector of vectors.filter((v:any)=>v.tool===id))expect(await action(frame,vector.action,vector.input)).toEqual(vector.output);
  if(['calculatrice','convertisseur-casse','timestamp-utc'].includes(String(id)))await expect(frame.getByRole('button',{name:'Refuser',exact:true})).toHaveCount(0);
 }
});
test('Cloudflare : le stockage non durable est refusé sans fausse confirmation',async({page})=>{
 const f=await open(page);
 expect(await f.evaluate(()=>navigator.storage.persisted())).toBe(false);
 await f.getByRole('button',{name:'Afficher les fichiers récents',exact:true}).click();
 await f.getByRole('button',{name:'Autoriser pour la session',exact:true}).click();
 await expect(f.locator('[data-node=notice]')).toContainText('UNAVAILABLE');
 await expect(f.locator('[data-node=recent]')).toHaveText('Aucun fichier récent affiché.');
});
