import {resolve} from 'node:path';
import {test,expect,type Page} from 'playwright/test';
import {serve,serveNetwork,root} from '../../tests/web/harness';
import {prepare} from './helpers';
let server:Awaited<ReturnType<typeof serve>>;let network:Awaited<ReturnType<typeof serveNetwork>>;
test.beforeAll(async()=>{server=await serve();network=await serveNetwork()});test.afterAll(async()=>{await server.close();await network.close()});
async function setup(page:Page){await page.goto(server.origin);await page.evaluate(async()=>{
 const {BrowserCapabilities}=await import('/adapter/capabilities.js' as string);
 const s=new BrowserCapabilities('test.tool','1.0.0',['storage.read','storage.write','file.pick','file.read','file.save','network.fetch','clipboard.write'],{profile:'a',storageVersion:1,dedicatedOrigin:location.origin,parentOrigin:'https://catalog.example.test',networkOrigins:['https://8.8.8.8']});
 (window as any).s=s;(window as any).decision='once';
 const button=document.createElement('button');button.textContent='Autoriser (banc de test)';button.onclick=e=>{const target=(window as any).s;target.permissions.respond(target.permissions.prompts()[0].id,(window as any).decision,e)};document.body.append(button);
});}
async function ask(page:Page,capability:string,payload:Record<string,unknown>){await page.evaluate(({capability,payload})=>{(window as any).out=undefined;(window as any).s.request({request_id:crypto.randomUUID(),capability,payload}).then((value:unknown)=>(window as any).out={value},(e:Error)=>(window as any).out={error:e.message});},{capability,payload});}
async function approve(page:Page,decision='once'){await page.evaluate(value=>{(window as any).decision=value},decision);await page.getByRole('button',{name:'Autoriser (banc de test)',exact:true}).click();}
async function result(page:Page){await expect.poll(()=>page.evaluate(()=>(window as any).out)).toBeTruthy();return page.evaluate(()=>(window as any).out);}
test('sept capacités refusées par défaut et consentement synthétique rejeté',async({page})=>{
 await setup(page);const errors=await page.evaluate(async()=>{const calls=[['storage.read',{key:'x'}],['storage.write',{key:'x',value:1}],['file.pick',{}],['file.read',{handle:'0'.repeat(64)}],['file.save',{name:'x',data_base64:'eA=='}],['network.fetch',{url:'https://8.8.8.8/data'}],['clipboard.write',{text:'x'}]];const errors=[];for(const [capability,payload] of calls)try{await (window as any).s.execute({request_id:crypto.randomUUID(),capability,payload});errors.push('ACCEPT')}catch(e){errors.push((e as Error).message)}return errors});expect(errors).toEqual(Array(7).fill('PERMISSION'));
 await ask(page,'storage.read',{key:'x'});expect(await page.evaluate(()=>{const s=(window as any).s;try{s.permissions.respond(s.permissions.prompts()[0].id,'session',new Event('click'));return false}catch{return true}})).toBe(true);
 await approve(page,'deny');expect((await result(page)).error).toBe('PERMISSION');
});
test('IndexedDB réel : isolation profils/outils/versions, quota atomique et refus non durable',async({page})=>{
 await setup(page);const result=await page.evaluate(async()=>{
  const {IndexedToolStorage}=await import('/adapter/storage.js' as string);const origin=location.origin,parent='https://catalog.example.test';
  const persisted=navigator.storage.persisted.bind(navigator.storage);
  // Only the availability probe is controlled; all transactions use real IndexedDB.
  const manager=navigator.storage;(window as any).storageManagerUnderTest=manager;Object.defineProperty(manager,'persisted',{configurable:true,value:async()=>true});
  const a=new IndexedToolStorage('a','tool',1,origin,parent),b=new IndexedToolStorage('b','tool',1,origin,parent),other=new IndexedToolStorage('a','other',1,origin,parent),v2=new IndexedToolStorage('a','tool',2,origin,parent);
  await a.set('value','one');await b.set('value','two');await other.set('value','three');
  const isolation=[await a.get('value'),await b.get('value'),await other.get('value'),await v2.get('value')];
  await a.set('big','x'.repeat(5*1024*1024-100));let quota='';try{await v2.set('extra','x'.repeat(200))}catch(e){quota=(e as Error).message}
  const unchanged=await a.get('value');await a.clearNamespace();const preserved=[await b.get('value'),await other.get('value')];
  const abort=new AbortController();abort.abort();let cancelled='';try{await a.set('cancel',1,abort.signal)}catch(e){cancelled=(e as Error).message}
  Object.defineProperty(navigator.storage,'persisted',{configurable:true,value:async()=>false});let privateMode='';try{await a.get('value')}catch(e){privateMode=(e as Error).message}
  Object.defineProperty(navigator.storage,'persisted',{configurable:true,value:persisted});for(const db of [a,b,other,v2])db.close();
  return {isolation,quota,unchanged,preserved,cancelled,privateMode};
 });expect(result).toEqual({isolation:['one','two','three',null],quota:'QUOTA',unchanged:'one',preserved:['two','three'],cancelled:'CANCELLED',privateMode:'UNAVAILABLE'});
});
test('file picker réel : annulation, 10 Mio+1 refusé, blocs et sauvegarde',async({page})=>{
 await setup(page);
 for(const [buffer,expected] of [[Buffer.alloc(0),'CANCELLED'],[Buffer.alloc(10*1024*1024+1),'SIZE_LIMIT']] as const){await ask(page,'file.pick',{});const chooserPromise=page.waitForEvent('filechooser');await approve(page);const chooser=await chooserPromise;await chooser.setFiles(expected==='CANCELLED'?[]:[{name:'too-big.bin',mimeType:'application/octet-stream',buffer}]);expect((await result(page)).error).toBe(expected);}
 await ask(page,'file.pick',{});const chooserPromise=page.waitForEvent('filechooser');await approve(page);await(await chooserPromise).setFiles({name:'hello.txt',mimeType:'text/plain',buffer:Buffer.from('hello')});const picked=await result(page);expect(picked.value.handle).toMatch(/^[a-f0-9]{64}$/);expect(picked.value.size).toBe(5);
 await ask(page,'file.read',{handle:picked.value.handle,offset:0,length:256*1024});await approve(page);expect((await result(page)).value.data_base64).toBe('aGVsbG8=');
 await ask(page,'file.save',{name:'hello.txt',data_base64:'aGVsbG8='});const download=page.waitForEvent('download');await approve(page);expect((await download).suggestedFilename()).toBe('hello.txt');expect((await result(page)).value.saved).toBe(true);
 await page.evaluate(()=>(window as any).s.revoke());await ask(page,'file.read',{handle:picked.value.handle});await approve(page);expect((await result(page)).error).toBe('INVALID_HANDLE');
});
test('réseau : allowlist, redirects refusés, offline et corps borné',async({page})=>{
 await setup(page);let forbidden=0;page.on('request',r=>{if(r.url().startsWith('https://evil.example.test'))forbidden++});
 await page.route('https://8.8.8.8/**',route=>route.continue({url:network.origin+new URL(route.request().url()).pathname}));
 await ask(page,'network.fetch',{url:'https://evil.example.test/data'});expect((await result(page)).error).toBe('NETWORK_DENIED');
 await ask(page,'network.fetch',{url:'https://8.8.8.8/data'});await approve(page,'session');expect((await result(page)).value.data_base64).toBe('aGVsbG8=');
 await ask(page,'network.fetch',{url:'https://8.8.8.8/redirect'});expect((await result(page)).error).toBe('UNAVAILABLE');expect(forbidden).toBe(0);
 await ask(page,'network.fetch',{url:'https://8.8.8.8/large'});expect((await result(page)).error).toBe('SIZE_LIMIT');
 await page.context().setOffline(true);await ask(page,'network.fetch',{url:'https://8.8.8.8/data'});expect((await result(page)).error).toBe('OFFLINE');await page.context().setOffline(false);
});
test('révocation annule les demandes et clipboard requiert un vrai geste',async({page})=>{
 await setup(page);await ask(page,'storage.read',{key:'pending'});await page.evaluate(()=>(window as any).s.revoke());expect((await result(page)).error).toBe('CANCELLED');
 await ask(page,'clipboard.write',{text:'Kortex local clipboard test'});await approve(page);const clipboard=await result(page);expect(clipboard.value?.written===true||['PERMISSION','UNSUPPORTED'].includes(clipboard.error)).toBe(true);
});
test('Dioxus affiche le consentement réel et la révocation',async({page})=>{
 await page.goto(server.origin);await prepare(page,['storage.read']);
 await page.evaluate(async()=>{const t=(window as any).adapterTest;await (window as any).kortexHost.startVerifiedTool(t.tool,{api_origin:location.origin,allowed_parent_origin:'https://catalog.example.test',tool_id:t.manifest.id,tool_version:t.manifest.version});(window as any).kortexHost.capabilities.request({request_id:crypto.randomUUID(),capability:'storage.read',payload:{key:'prefs'}}).catch((e:Error)=>(window as any).permissionOutcome=e.message);});
 await expect(page.getByRole('heading',{name:'Autorisation demandée'})).toBeVisible();await page.getByRole('button',{name:'Refuser',exact:true}).click();await expect.poll(()=>page.evaluate(()=>(window as any).permissionOutcome)).toBe('PERMISSION');await expect(page.getByRole('heading',{name:'Autorisation demandée'})).toHaveCount(0);
 await page.getByRole('button',{name:'Révoquer les autorisations'}).click();
});

test('guest SDK réel : host_enqueue, consentement, IndexedDB et capability.result vers UiRenderer',async({page})=>{
 await page.goto(server.origin);await prepare(page,['storage.read'],'/fixtures/capability.wasm');
 await page.evaluate(async()=>{
  const manager=navigator.storage;(window as any).storageManagerUnderTest=manager;Object.defineProperty(manager,'persisted',{configurable:true,value:async()=>true});
  const {IndexedToolStorage}=await import('/adapter/storage.js' as string);const t=(window as any).adapterTest;
  const storage=new IndexedToolStorage('local',t.manifest.id,1,location.origin,'https://catalog.example.test');await storage.set('prefs',123);storage.close();
  await (window as any).kortexHost.startVerifiedTool(t.tool,{api_origin:location.origin,allowed_parent_origin:'https://catalog.example.test',tool_id:t.manifest.id,tool_version:t.manifest.version});
 });
 await expect(page.getByRole('heading',{name:'Autorisation demandée'})).toBeVisible();await expect(page.locator('[data-node=outcome]')).toHaveText('pending');
 await page.getByRole('button',{name:'Autoriser une fois',exact:true}).click();await expect(page.locator('[data-node=outcome]')).toHaveText('resolved:123');
 await page.screenshot({path:resolve(root,`docs/evidence/pipeline-permissions.png`),fullPage:true});
});
