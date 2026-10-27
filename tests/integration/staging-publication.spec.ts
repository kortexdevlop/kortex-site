import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {serve,root} from '../web/harness';
const gate=JSON.parse(readFileSync(resolve(root,'target/staging-cloud/report.json'),'utf8'));
if(!gate.cloud_roundtrip_passed||!gate.api_origin?.startsWith('https://kortex-api-staging.'))throw Error('A successful real staging roundtrip is required; this gate cannot be mocked or skipped');
const descriptor=JSON.parse(readFileSync(gate.descriptor,'utf8')),bundle=JSON.parse(readFileSync(gate.host_trust,'utf8'));
let server:Awaited<ReturnType<typeof serve>>;
test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>server.close());
test('real D1/R2 downloaded package is verified and started by the Web host',async({page},info)=>{
 const catalog=await fetch(gate.api_origin+'/api/v1/tools/calculatrice');expect(catalog.status).toBe(200);const detail=await catalog.json();expect(detail.descriptor).toEqual(descriptor);
 const response=await fetch(gate.api_origin+'/api/v1/downloads/calculatrice/1.0.0');expect(response.status).toBe(200);expect(response.headers.get('content-type')).toBe('application/vnd.kortex.tool+zip');const bytes=Buffer.from(await response.arrayBuffer());expect(createHash('sha256').update(bytes).digest('hex')).toBe(gate.sha256);expect(bytes).toEqual(readFileSync(gate.archive));
 expect((await fetch(gate.api_origin+'/api/v1/downloads/calculatrice/'+gate.revoked_version)).status).toBe(410);
 // Serve precisely those network-downloaded bytes through the local host harness; no staging website exists yet.
 await page.route(server.origin+'/fixtures/cloud.tool',route=>route.fulfill({contentType:'application/vnd.kortex.tool+zip',body:bytes}));await page.goto(server.origin);
 await page.evaluate(async f=>{const api=await import('/adapter/interop.js' as string);await (window as any).kortexHost.loadTool({...f.descriptor,url:location.origin+'/fixtures/cloud.tool'},api.TrustStore.fromSignedKeys(f.bundle.document,f.bundle.root,f.bundle.minimum_version),{api_origin:location.origin,allowed_parent_origin:location.origin,tool_id:'calculatrice',tool_version:'1.0.0'})},{descriptor,bundle});
 await expect(page.locator('[data-node=result]')).toHaveText('Prêt à calculer');
 await page.evaluate(async()=>{await (window as any).kortexHost.command({kind:'action.invoke',payload:{action_id:'multiply',input:{a:25,b:48}}})});await expect(page.getByLabel('Résultat',{exact:true})).toContainText('1200');
 await page.screenshot({path:resolve(root,`docs/evidence/staging9-${info.project.name}.png`),fullPage:true});
 await page.getByRole('button',{name:'Fermer l’outil',exact:true}).click();await expect(page.locator('[data-node=calculator]')).toHaveCount(0);
});
