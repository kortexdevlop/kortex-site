// Allowlisted metadata only. Never inspect bodies, query strings, headers,
// provider output, object keys or exception messages.
export type Route='health'|'catalog'|'download'|'publication'|'identity'|'account'|'workflow'|'ai'|'runtime'|'trust'|'other';
export type TelemetryRow={v:1;event:'request'|'dependency';environment:'local'|'staging'|'production';request_id:string;minute:number;route:Route;status:number;duration_ms:number;service?:'r2'|'workers_ai';operation?:'get'|'head'|'put'|'run'};
export function routeName(path:string):Route {
 if(path==='/api/v1/health')return 'health';
 for(const [prefix,name] of [['/api/v1/tools','catalog'],['/api/v1/downloads/','download'],['/api/v1/admin/','publication'],['/api/v1/auth/','identity'],['/api/v1/me/workflows','workflow'],['/api/v1/me/','account'],['/api/v1/ai/','ai'],['/api/v1/runtime','runtime'],['/api/v1/trust/','trust']] as const)if(path.startsWith(prefix))return name;
 return 'other';
}
export type TelemetrySink=(row:TelemetryRow)=>void;
export function telemetry(request:Request,bindings:unknown,sink:TelemetrySink=row=>console.log(JSON.stringify(row))){
 const raw=bindings&&typeof bindings==='object'?bindings as Record<string,unknown>:{};
 const enabled=raw.TELEMETRY_ENABLED==='true';
 const environment:TelemetryRow['environment']=raw.KORTEX_ENV==='staging'?'staging':raw.KORTEX_ENV==='production'?'production':'local';
 // Generated independently of client headers, which can encode identifiers.
 const id=crypto.randomUUID(),route=routeName(new URL(request.url).pathname),start=performance.now();
 const emit=(row:TelemetryRow)=>{if(enabled)try{sink(row)}catch{/* Logging cannot break a user request. */}};
 const base={v:1 as const,environment,request_id:id,minute:Math.floor(Date.now()/60000),route};
 const instrument=(target:unknown,service:'r2'|'workers_ai',allowed:string[])=>{
  if(!enabled||!target||typeof target!=='object')return target;
  return new Proxy(target,{get(object,key){const value=Reflect.get(object,key,object);if(typeof value!=='function')return value;
   if(!allowed.includes(String(key)))return value.bind(object);
   return async(...args:unknown[])=>{const before=performance.now();let status=200;try{return await value.apply(object,args)}catch(error){status=503;throw error}finally{emit({...base,event:'dependency',service,operation:key as TelemetryRow['operation'],status,duration_ms:Math.max(0,Math.round(performance.now()-before))})}};
  }});
 };
 return {id,bindings:enabled?{...raw,TOOLS_BUCKET:instrument(raw.TOOLS_BUCKET,'r2',['get','head','put']),AI:instrument(raw.AI,'workers_ai',['run'])}:bindings,
  finish(response:Response){emit({...base,event:'request',status:response.status,duration_ms:Math.max(0,Math.round(performance.now()-start))});if(!enabled)return response;const headers=new Headers(response.headers);headers.set('X-Kortex-Trace-ID',id);return new Response(response.body,{status:response.status,statusText:response.statusText,headers})}};
}

// A bounded five-minute window. No request IDs or other dimensions retained.
// This is a deterministic evaluator, not a distributed/global alert collector.
export class HealthWindow {
 private minutes=new Map<number,{requests:number;errors:number;limited:number;latency:number[]}>();
 private latestMinute=-Infinity;
 add(status:number,duration:number,now=Date.now()){
  const minute=Math.floor(now/60000);this.latestMinute=Math.max(this.latestMinute,minute);if(minute<this.latestMinute-4)return;for(const key of this.minutes.keys())if(key<this.latestMinute-4)this.minutes.delete(key);
  const row=this.minutes.get(minute)??{requests:0,errors:0,limited:0,latency:[]};row.requests++;if(status>=500)row.errors++;if(status===429)row.limited++;
  if(Number.isFinite(duration)&&row.latency.length<1024)row.latency.push(Math.max(0,duration));this.minutes.set(minute,row);
 }
 snapshot(now=Date.now()){
  const minute=Math.floor(now/60000),rows=[...this.minutes].filter(([key])=>key>=minute-4&&key<=minute).map(([,v])=>v),requests=rows.reduce((n,r)=>n+r.requests,0),errors=rows.reduce((n,r)=>n+r.errors,0),limited=rows.reduce((n,r)=>n+r.limited,0),latency=rows.flatMap(r=>r.latency).sort((a,b)=>a-b);
  return {requests,errors,limited,error_rate:requests?errors/requests:0,p95_ms:latency.length?latency[Math.ceil(latency.length*.95)-1]:0,latency_samples:latency.length,alert:requests>=100&&errors/requests>.02};
 }
}
