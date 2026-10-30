import {mkdirSync,readFileSync,writeFileSync,existsSync,rmdirSync} from 'node:fs';
import {resolve} from 'node:path';
export const DRILL_LIMITS=Object.freeze({reads:2000,writes:1000,queries:70,requests:60,attemptsPerUtcDay:3});
export function d1Budget(limits=DRILL_LIMITS){
 const usage={rows_read:0,rows_written:0,query_calls:0};
 return {usage,before(){if(usage.query_calls>=limits.queries||usage.rows_read>=limits.reads||usage.rows_written>=limits.writes)throw Error('DRILL_D1_BUDGET_EXHAUSTED');usage.query_calls++},record(results){for(const r of results){const m=r.meta;if(!m||!Number.isSafeInteger(m.rows_read)||!Number.isSafeInteger(m.rows_written)||m.rows_read<0||m.rows_written<0)throw Error('DRILL_D1_METERING_REQUIRED');usage.rows_read+=m.rows_read;usage.rows_written+=m.rows_written}if(usage.rows_read>limits.reads||usage.rows_written>limits.writes)throw Error('DRILL_D1_BUDGET_EXCEEDED')}};
}
// Local execution fuse, not an account-wide or distributed billing guarantee.
export function reserveDrillAttempt(root,account,now=new Date()){
 const dir=resolve(root,'target/d1-qa-budget');mkdirSync(dir,{recursive:true});const file=resolve(dir,now.toISOString().slice(0,10)+'-'+account+'.json');
 const lock=file+'.lock';try{mkdirSync(lock)}catch{throw Error('DRILL_BUDGET_LOCKED')}
 try{const old=existsSync(file)?JSON.parse(readFileSync(file,'utf8')):{attempts:0};if(!Number.isSafeInteger(old.attempts)||old.attempts<0||old.attempts>=DRILL_LIMITS.attemptsPerUtcDay)throw Error('DRILL_DAILY_ATTEMPT_LIMIT');
 writeFileSync(file,JSON.stringify({attempts:old.attempts+1,reserved_reads:(old.attempts+1)*DRILL_LIMITS.reads,reserved_writes:(old.attempts+1)*DRILL_LIMITS.writes})+'\n');return old.attempts+1;
 }finally{rmdirSync(lock)}
}
