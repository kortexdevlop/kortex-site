import assert from 'node:assert/strict';
export const percentiles=values=>{const a=[...values].sort((x,y)=>x-y);return{count:a.length,p50_ms:a[Math.ceil(a.length*.5)-1],p95_ms:a[Math.ceil(a.length*.95)-1],max_ms:a.at(-1)}};
export async function catalogLoad(get){
 const ids=new Set();let cursor=null,pages=0;
 do{const p=await get('/api/v1/tools?limit=100'+(cursor?'&cursor='+encodeURIComponent(cursor):''));assert.equal(p.status,200);for(const t of p.body.items){assert(!ids.has(t.id),'duplicate ID');ids.add(t.id)}cursor=p.body.next_cursor;pages++;assert(pages<=100)}while(cursor);
 assert.equal(ids.size,10000);for(let n=1;n<=10000;n++)assert(ids.has('qa-tool-'+String(n).padStart(5,'0')));
 const filters={};for(const [category,count]of [['calculation',9990],['developer',9],['data',1]]){let total=0,cursor=null;do{const p=await get('/api/v1/tools?limit=100&category='+category+(cursor?'&cursor='+encodeURIComponent(cursor):''));assert.equal(p.status,200);assert(p.body.items.every(i=>i.category===category));total+=p.body.items.length;cursor=p.body.next_cursor}while(cursor);assert.equal(total,count);filters[category]=total}
 const first=await get('/api/v1/tools?limit=8&sort=name&tag=qa&q=qa%20tool');assert.equal(first.body.items.length,8);assert(first.body.next_cursor);assert.equal((await get('/api/v1/tools?limit=8&category=data&cursor='+encodeURIComponent(first.body.next_cursor))).status,400);
 for(const query of ['limit=101','limit=8&limit=9','cursor=invalid','unknown=1'])assert.equal((await get('/api/v1/tools?'+query)).status,400);
 const measurements={};for(const [kind,path]of [['list','/api/v1/tools?limit=20&q=qa%20tool&tag=qa'],['detail','/api/v1/tools/qa-tool-05000']]){for(let i=0;i<5;i++)assert.equal((await get(path)).status,200);const start=performance.now(),samples=await Promise.all(Array.from({length:100},async(_,i)=>{await new Promise(r=>setTimeout(r,Math.max(0,start+i*100-performance.now())));const p=await get(path);assert.equal(p.status,200);return p.ms}));measurements[kind]={...percentiles(samples),samples_ms:samples,scheduled_rps:10}}
 return{unique_ids:ids.size,pages,filters,measurements,thresholds_met:measurements.list.p95_ms<=500&&measurements.detail.p95_ms<=300};
}
