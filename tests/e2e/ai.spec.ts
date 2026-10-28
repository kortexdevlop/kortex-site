import {test,expect} from 'playwright/test';
import {writeFileSync} from 'node:fs';
const web='https://kortex-web-staging.userpgl.workers.dev';
const make=(tool_id:string,action_id:string,input:unknown)=>({tool_id,action_id,input,requires_consent:true});
test('assistant reviews four public tools, executes real signed WASM only after approval',async({page})=>{
 const steps=[make('calculatrice','multiply',{a:25,b:48}),make('timestamp-utc','to_unix',{iso_utc:'2024-01-01T00:00:00Z'}),make('convertisseur-casse','upper',{text:'Été ß'}),make('explorateur-csv','summarize_csv',{data_text:'a,b\n1,2\n3,4\n'})];let frames=0,aiCalls=0;
 // Deterministic proposal transport ONLY in this test; catalogue, admission, WASM and capability policy are real staging.
 await page.route('**/api/v1/me',route=>route.fulfill({json:{user:{id:'ai-ui-test',sync_enabled:false},csrf_token:'test-only'}}));
 await page.route('**/api/v1/ai/plan',route=>{aiCalls++;return route.fulfill({json:{v:1,summary:'Quatre propositions publiques de test',steps}})});
 page.on('request',request=>{if(new URL(request.url()).pathname==='/frame.html')frames++});
 await page.goto('/ai');await expect(page.getByRole('button',{name:'Essayer un exemple local',exact:true})).toBeEnabled();await page.getByLabel('Votre demande',{exact:true}).fill('Proposer quatre actions publiques');await page.getByLabel(/J’autorise l’envoi/).check();await page.getByRole('button',{name:'Demander une proposition',exact:true}).click();await expect(page.getByRole('heading',{name:'Suggestion IA',exact:true})).toBeVisible();expect(frames).toBe(0);expect(aiCalls).toBe(1);
 const outputs=[];for(let i=0;i<steps.length;i++){await page.getByRole('button',{name:'Approuver l’action '+(i+1),exact:true}).click();await expect(page.locator('[data-ai-state="'+i+'"]')).toHaveText('Exécutée localement');const text=await page.locator('[data-ai-result="'+i+'"]').innerText();outputs.push(JSON.parse(text));expect(aiCalls).toBe(1)}
 expect(outputs[0].result).toBe(1200);expect(outputs[1].unix_seconds).toBe(1704067200);expect(outputs[2].text).toBe('ÉTÉ SS');expect(outputs[3].rows).toBe(2);expect(frames).toBe(4);
 await page.setViewportSize({width:1360,height:950});await page.screenshot({path:'docs/evidence/assistant-web.png',fullPage:true});
 writeFileSync('docs/evidence/assistant-four-tools.json',JSON.stringify({date:new Date().toISOString(),passed:true,proposal_provider:'test-only deterministic route',real_signed_staging_runtime:true,ai_calls:aiCalls,frames,outputs},null,2)+'\n');
});
test('refusal, malformed plan, unknown action, quota and logout never execute',async({page})=>{
 let status=200,body:any={v:1,summary:'Refus',steps:[make('calculatrice','multiply',{a:25,b:48})]},signedIn=true,frames=0;
 await page.route('**/api/v1/me',route=>route.fulfill({status:signedIn?200:401,json:{user:{id:'ai-test'},csrf_token:'test-only'}}));await page.route('**/api/v1/ai/plan',route=>route.fulfill({status,json:body}));page.on('request',r=>{if(new URL(r.url()).pathname==='/frame.html')frames++});await page.goto('/ai');await expect(page.getByRole('button',{name:'Essayer un exemple local',exact:true})).toBeEnabled();await page.getByLabel('Votre demande',{exact:true}).fill('Proposition de test');await page.getByLabel(/J’autorise l’envoi/).check();const send=page.getByRole('button',{name:'Demander une proposition',exact:true});await send.click();await page.getByRole('button',{name:'Refuser l’action 1',exact:true}).click();await expect(page.locator('[data-ai-state="0"]')).toHaveText('Refusée');expect(frames).toBe(0);
 body={bad:true};await send.click();await expect(page.getByRole('status')).toContainText('indisponible');expect(frames).toBe(0);
 body={v:1,summary:'Action inconnue',steps:[make('calculatrice','delete-files',{})]};await send.click();await page.getByRole('button',{name:'Approuver l’action 1',exact:true}).click();await expect(page.locator('[data-ai-state="0"]')).toContainText('Échec');expect(frames).toBe(0);
 status=429;await send.click();await expect(page.getByRole('status')).toContainText('Quota quotidien');expect(frames).toBe(0);
 status=200;body={v:1,summary:'Déconnexion',steps:[make('calculatrice','multiply',{a:25,b:48})]};await send.click();await expect(page.getByRole('heading',{name:'Suggestion IA',exact:true})).toBeVisible();signedIn=false;await page.getByRole('button',{name:'Approuver l’action 1',exact:true}).click();await expect(page.locator('[data-ai-state="0"]')).toContainText('Échec');expect(frames).toBe(0);
});
