import {it,expect,vi} from 'vitest';
import {telemetry,HealthWindow,routeName,type TelemetryRow} from '../src/lib/telemetry';
import {handle} from '../src/index';
import {env} from 'cloudflare:workers';
it('only emits bounded anonymous metadata, even for credentials, private files and provider failures',async()=>{
 const privateData='PERSONAL_FILE_admin@example.test_C:\\private\\photo.jpg_Bearer_secret',rows:TelemetryRow[]=[];
 const input=new Request('https://test/api/v1/downloads/'+privateData+'?token='+privateData,{method:'POST',headers:{Authorization:privateData,'X-Request-ID':'99999999-0000-4000-8000-000000000001'},body:privateData});
 const obs=telemetry(input,{KORTEX_ENV:'staging',TELEMETRY_ENABLED:'true',TOOLS_BUCKET:{async get(_key:string){throw Error(privateData)}},AI:{async run(){return {privateData}}}},r=>rows.push(r));
 const b=obs.bindings as {TOOLS_BUCKET:{get(k:string):Promise<unknown>};AI:{run(...args:unknown[]):Promise<unknown>}};
 await expect(b.TOOLS_BUCKET.get(privateData)).rejects.toThrow(privateData);await b.AI.run(privateData,{prompt:privateData});obs.finish(new Response(privateData,{status:503}));
 expect(rows).toHaveLength(3);expect(JSON.stringify(rows)).not.toContain(privateData);expect(JSON.stringify(rows)).not.toContain('99999999');expect(rows.every(r=>r.request_id===obs.id&&r.route==='download')).toBe(true);expect(rows[0].status).toBe(503);expect(rows[2].status).toBe(503);
});
it('disabled telemetry and sink failures preserve service behavior',async()=>{
 const sink=vi.fn(),obs=telemetry(new Request('https://test/secret'),{TELEMETRY_ENABLED:'false'},sink);expect(obs.finish(new Response('ok')).status).toBe(200);expect(sink).not.toHaveBeenCalled();
 expect(telemetry(new Request('https://test'),{TELEMETRY_ENABLED:'true'},()=>{throw Error('collector unavailable')}).finish(new Response('ok')).status).toBe(200);
});
it('integrates correlation and redaction with the actual Worker handler',async()=>{
 const log=vi.spyOn(console,'log').mockImplementation(()=>{});try{const r=await handle(new Request('https://test/api/v1/health?private=secret',{headers:{Authorization:'Bearer PRIVATE_FILE'}}),{...env,TELEMETRY_ENABLED:'true'});expect(r.status).toBe(200);expect(log).toHaveBeenCalledTimes(1);const row=JSON.parse(log.mock.calls[0][0]);expect(row.request_id).toBe(r.headers.get('X-Kortex-Trace-ID'));expect(row.route).toBe('health');expect(JSON.stringify(row)).not.toMatch(/secret|PRIVATE_FILE|Authorization/)}finally{log.mockRestore()}
});
it('alerts above 2% errors on at least 100 requests in five minutes and expires the window',()=>{
 const h=new HealthWindow(),now=Date.now();for(let n=0;n<99;n++)h.add(n<3?503:n===3?429:200,n,now);expect(h.snapshot(now).alert).toBe(false);h.add(200,100,now);expect(h.snapshot(now)).toMatchObject({requests:100,errors:3,limited:1,alert:true});expect(h.snapshot(now+300000).requests).toBe(0);
 const healthy=new HealthWindow();for(let n=0;n<100;n++)healthy.add(n<2?500:200,10,now);expect(healthy.snapshot(now).alert).toBe(false);expect(routeName('/some/private/email')).toBe('other');
 const shuffled=new HealthWindow();shuffled.add(503,10,now);shuffled.add(200,20,now-60000);expect(shuffled.snapshot(now).requests).toBe(2);shuffled.add(200,20,now-600000);expect(shuffled.snapshot(now).requests).toBe(2);
});
