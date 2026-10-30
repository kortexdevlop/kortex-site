import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';import {resolve} from 'node:path';import {pathToFileURL} from 'node:url';
export async function probeProduction(config,fetcher=fetch){
 if(config.enabled!==true)return{enabled:false,passed:false,reason:'PRODUCTION_MONITOR_NOT_ACTIVATED',requests:0,d1_queries:0};
 if(config.max_requests_per_run!==3||config.targets?.length!==3)throw Error('THREE_PROBE_LIMIT_REQUIRED');
 const results=[];
 for(const target of config.targets){const url=new URL(target.url);if(url.protocol!=='https:'||url.username||url.password||url.search||url.hash||!['kortex.site','api.kortex.site','run.kortex.site'].includes(url.hostname)||!['/api/v1/health','/health.json'].includes(url.pathname))throw Error('UNSAFE_MONITOR_TARGET');}
 for(const target of config.targets){try{const r=await fetcher(target.url,{redirect:'error',cache:'no-store',signal:AbortSignal.timeout(10000)}),j=await r.json();results.push({service:target.service,status:r.status,passed:r.ok&&j.status==='ok'&&(target.service==='kortex-api'?j.api_version==='1':j.service===target.service)})}catch{results.push({service:target.service,passed:false,status:null})}}
 return{enabled:true,passed:results.every(x=>x.passed),requests:results.length,d1_queries:0,scope:'Static health surfaces only; not catalogue, login or end-to-end availability',results};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){const config=JSON.parse(readFileSync('infra/production-monitor.json')),result=await probeProduction(config);mkdirSync('target/ci-reports',{recursive:true});writeFileSync('target/ci-reports/production-monitor.json',JSON.stringify(result,null,2)+'\n');console.log(JSON.stringify(result));if(result.enabled&&!result.passed)process.exitCode=1}
