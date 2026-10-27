import {test,expect} from 'playwright/test';
import {readFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {root,serve} from '../../tests/web/harness';
let server:Awaited<ReturnType<typeof serve>>;
test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>server.close());
test('six paquets correctement signés mais malformés refusés sans création de Worker',async({page})=>{
 const corpus=JSON.parse(readFileSync(resolve(root,'target/malformed-admission.json'),'utf8'));
 await page.goto(server.origin);
 const result=await page.evaluate(async(corpus)=>{
  const api=await import('/adapter/interop.js' as string);
  const trust=new api.TrustStore([{id:'hostile-test',public_key_base64:corpus.public_key_base64,not_before:0,expires:4102444800000,revoked:false}]);
  let workers=0;const Original=window.Worker;
  window.Worker=class extends Original {constructor(...args:any[]){workers++;super(args[0],args[1])}};
  try{const refused=[];for(const c of corpus.cases){const bytes=Uint8Array.from(atob(c.archive_base64),(v)=>v.charCodeAt(0));await trust.verify(bytes,c.descriptor);try{await api.VerifiedTool.verifyArchive(bytes,c.descriptor,trust);throw Error('ACCEPTED_HOSTILE_PACKAGE')}catch(e){if((e as Error).message==='ACCEPTED_HOSTILE_PACKAGE')throw e;refused.push(c.name)}}return {refused,workers}}finally{window.Worker=Original}
 },corpus);
 expect(result.refused).toEqual(corpus.cases.map((c:any)=>c.name));expect(result.workers).toBe(0);
});
