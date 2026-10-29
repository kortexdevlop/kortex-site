import {HealthWindow} from '../api/src/lib/telemetry.ts';
import {createInterface} from 'node:readline';
import {createReadStream} from 'node:fs';
const source=process.argv[2];if(!source)throw Error('Usage: node scripts/health-summary.mjs telemetry.ndjson');
const window=new HealthWindow(),now=Date.now();let accepted=0,rejected=0,r2=0,ai=0;
for await(const line of createInterface({input:createReadStream(source),crlfDelay:Infinity})){
 if(line.length>16384){rejected++;continue}let row;try{row=JSON.parse(line)}catch{rejected++;continue}
 if(row.v!==1||!['request','dependency'].includes(row.event)||!Number.isSafeInteger(row.minute)||row.minute<Math.floor(now/60000)-4||row.minute>Math.floor(now/60000)||!Number.isInteger(row.status)||row.status<100||row.status>599||!Number.isFinite(row.duration_ms)||row.duration_ms<0){rejected++;continue}
 if(row.event==='request'){window.add(row.status,row.duration_ms,row.minute*60000);accepted++}else{if(row.service==='r2')r2++;if(row.service==='workers_ai')ai++}
}
const snapshot=window.snapshot(now);console.log(JSON.stringify({date:new Date(now).toISOString(),window_minutes:5,...snapshot,accepted,rejected,r2_operations:r2,ai_provider_calls:ai,notification_sent:false,scope:'input file only; operations are not billable neurons, tokens, bytes or global account costs'}));
if(snapshot.alert)process.exitCode=2;
