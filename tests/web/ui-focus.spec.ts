import {test,expect} from 'playwright/test';
import {serve} from './harness';
let server:Awaited<ReturnType<typeof serve>>;
test.beforeAll(async()=>{server=await serve()});test.afterAll(async()=>server.close());
test('focus réel du renderer partagé avec flèches, Home et End',async({page})=>{
 await page.addInitScript(()=>{
  const leaf=(key:string,text:string)=>({type:'text',key,props:{text},children:[]});const ui={type:'tabs',key:'sections',props:{label:'Sections'},children:[leaf('one','Premier'),leaf('two','Second')]};
  (window as any).kortexHost={snapshot:()=>({status:'ready',ui}),command:async()=>{}};
 });
 await page.goto(server.origin);const first=page.getByRole('tab',{name:'Premier'}),second=page.getByRole('tab',{name:'Second'});
 await first.focus();await page.keyboard.press('ArrowRight');await expect(second).toBeFocused();await expect(second).toHaveAttribute('aria-selected','true');await expect(page.getByRole('tabpanel',{name:'Second'})).toBeVisible();
 await page.keyboard.press('Home');await expect(first).toBeFocused();await page.keyboard.press('End');await expect(second).toBeFocused();
});
