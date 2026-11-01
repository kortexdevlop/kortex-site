import {test,expect} from 'playwright/test';
import {writeFileSync} from 'node:fs';
const web='https://kortex-web-staging.userpgl.workers.dev';
const make=(tool_id:string,action_id:string,input:unknown,bind={})=>({tool_id,action_id,input,bind,requires_consent:true});
test('assistant reviews four public tools, executes real signed WASM only after approval',async({page})=>{
 const steps=[make('calculatrice','multiply',{a:25,b:48}),make('timestamp-utc','to_unix',{iso_utc:'2024-01-01T00:00:00Z'}),make('convertisseur-casse','upper',{text:'Été ß'}),make('explorateur-csv','summarize_csv',{data_text:'a,b\n1,2\n3,4\n'})];let frames=0,aiCalls=0;
 // Deterministic proposal transport ONLY in this test; catalogue, admission, WASM and capability policy are real staging.
 await page.route('**/api/v1/me',route=>route.fulfill({json:{user:{id:'ai-ui-test',sync_enabled:false},csrf_token:'test-only'}}));
 await page.route('**/api/v1/ai/chat',route=>{aiCalls++;return route.fulfill({json:{v:2,reply:'Quatre propositions publiques de test',steps}})});
 page.on('request',request=>{if(new URL(request.url()).pathname==='/frame.html')frames++});
 await page.goto('/ai');await expect(page.getByRole('button',{name:'Essayer un exemple local',exact:true})).toBeEnabled();await page.getByLabel('Votre message',{exact:true}).fill('Proposer quatre actions publiques');await page.getByLabel(/J’autorise l’envoi/).check();await page.getByRole('button',{name:'Envoyer',exact:true}).click();await expect(page.getByRole('heading',{name:'4 actions à la suite',exact:true})).toBeVisible();expect(frames).toBe(0);expect(aiCalls).toBe(1);await page.getByRole('button',{name:'Lancer les 4 actions',exact:true}).click();
 const outputs=[];for(let i=0;i<steps.length;i++){await expect(page.locator('[data-ai-state="'+i+'"]')).toHaveText('Exécutée localement');const text=await page.locator('[data-ai-result="'+i+'"]').innerText();outputs.push(JSON.parse(text));expect(aiCalls).toBe(1)}
 expect(outputs[0].result).toBe(1200);expect(outputs[1].unix_seconds).toBe(1704067200);expect(outputs[2].text).toBe('ÉTÉ SS');expect(outputs[3].rows).toBe(2);expect(frames).toBe(4);
 await page.setViewportSize({width:1360,height:950});await page.screenshot({path:'docs/evidence/assistant-web.png',fullPage:true});
 writeFileSync('docs/evidence/assistant-four-tools.json',JSON.stringify({date:new Date().toISOString(),passed:true,proposal_provider:'test-only deterministic route',real_signed_staging_runtime:true,ai_calls:aiCalls,frames,outputs},null,2)+'\n');
});
test('refusal, malformed plan, unknown action, quota and logout never execute',async({page})=>{
 let status=200,body:any={v:2,reply:'Refus',steps:[make('calculatrice','multiply',{a:25,b:48})]},signedIn=true,frames=0;
 await page.route('**/api/v1/me',route=>route.fulfill({status:signedIn?200:401,json:{user:{id:'ai-test'},csrf_token:'test-only'}}));await page.route('**/api/v1/ai/chat',route=>route.fulfill({status,json:body,headers:status===429?{'X-Kortex-Limit-Source':'application'}:{}}));page.on('request',r=>{if(new URL(r.url()).pathname==='/frame.html')frames++});await page.goto('/ai');await expect(page.getByRole('button',{name:'Essayer un exemple local',exact:true})).toBeEnabled();await page.getByLabel('Votre message',{exact:true}).fill('Proposition de test');await page.getByLabel(/J’autorise l’envoi/).check();const send=page.getByRole('button',{name:'Envoyer',exact:true}),ask=async()=>{await page.getByLabel('Votre message',{exact:true}).fill('Proposition de test');await send.click()};await send.click();await page.getByRole('button',{name:'Refuser',exact:true}).last().click();await expect(page.locator('[data-ai-state="0"]').last()).toHaveText('Refusée');expect(frames).toBe(0);
 body={bad:true};await ask();await expect(page.locator('.ai-notice')).toContainText('indisponible');expect(frames).toBe(0);
 body={v:2,reply:'Action inconnue',steps:[make('calculatrice','delete-files',{})]};await ask();await page.getByRole('button',{name:/^Lancer/}).last().click();await expect(page.locator('[data-ai-state="0"]').last()).toContainText('Échec');expect(frames).toBe(0);
 status=429;await ask();await expect(page.locator('.ai-notice')).toContainText('crédits IA');expect(frames).toBe(0);
 status=200;body={v:2,reply:'Déconnexion',steps:[make('calculatrice','multiply',{a:25,b:48})]};await send.click();await expect(page.getByText('Déconnexion',{exact:true})).toBeVisible();await expect(page.getByRole('heading',{name:'Action proposée',exact:true}).last()).toBeVisible();signedIn=false;await page.getByRole('button',{name:/^Lancer/}).last().click();await expect(page.locator('[data-ai-state="0"]').last()).toContainText('Échec');expect(frames).toBe(0);
});
