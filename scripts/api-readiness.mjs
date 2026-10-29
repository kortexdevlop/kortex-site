export async function waitForApi(env,c,{fetcher=fetch,delay=ms=>new Promise(r=>setTimeout(r,ms)),attempts=20}={}){
 if(env.GITHUB_REPOSITORY!=='kortexdevlop/kortex-site'||!/^[a-f0-9]{40}$/.test(env.GITHUB_SHA||''))throw Error('UNTRUSTED_API_DEPLOY_CONTEXT');
 const headers={Authorization:'Bearer '+env.GITHUB_TOKEN,Accept:'application/vnd.github+json'};
 for(let attempt=0;attempt<attempts;attempt++){
  const runs=await fetcher(`https://api.github.com/repos/${env.GITHUB_REPOSITORY}/actions/workflows/api.yml/runs?head_sha=${env.GITHUB_SHA}&per_page=20`,{headers,redirect:'error',signal:AbortSignal.timeout(15000)});if(!runs.ok)throw Error('API_DEPLOY_STATUS_UNAVAILABLE');const jobs=(await runs.json()).workflow_runs??[];
  if(jobs.some(r=>r.status!=='completed')){await delay(30000);continue}if(jobs.some(r=>r.conclusion==='failure')&&!jobs.some(r=>r.conclusion==='success'))throw Error('API_DEPLOYMENT_FAILED');
  const health=await fetcher(c.api_origin+'/api/v1/health',{cache:'no-store',redirect:'error',signal:AbortSignal.timeout(15000)});if(!health.ok||(await health.json()).api_version!=='1'){await delay(30000);continue}
  const revision=health.headers.get('x-kortex-api-revision');if(revision===env.GITHUB_SHA)return true;if(!/^[a-f0-9]{40}$/.test(revision||'')){await delay(30000);continue}
  const compare=await fetcher(`https://api.github.com/repos/${env.GITHUB_REPOSITORY}/compare/${revision}...${env.GITHUB_SHA}`,{headers,redirect:'error',signal:AbortSignal.timeout(15000)});if(!compare.ok)throw Error('API_COMPATIBILITY_UNAVAILABLE');const diff=await compare.json();
  if(!['ahead','identical'].includes(diff.status)||!Array.isArray(diff.files)||diff.files.length>=300||diff.files.some(f=>/^(api\/|database\/|shared\/|web-adapter\/src\/(?:archive|wasm-check)\.ts$)/.test(f.filename)))throw Error('API_DEPLOYMENT_REQUIRED');
  return true;
 }throw Error('API_NOT_READY');
}
