import {resolve} from 'node:path';import {pathToFileURL} from 'node:url';
import {channels} from './ci-publish-tool.mjs';
export async function requireQuality(env,fetcher=fetch,delay=ms=>new Promise(r=>setTimeout(r,ms))){
 if(env.GITHUB_REPOSITORY!==channels.repository||env.GITHUB_REF!==channels.branch||!/^[a-f0-9]{40}$/.test(env.GITHUB_SHA||''))throw Error('UNTRUSTED_QUALITY_CONTEXT');
 for(let attempt=0;attempt<90;attempt++){
  const response=await fetcher(`https://api.github.com/repos/${channels.repository}/actions/workflows/validate.yml/runs?head_sha=${env.GITHUB_SHA}&per_page=20`,{headers:{Authorization:'Bearer '+env.GITHUB_TOKEN,Accept:'application/vnd.github+json'},redirect:'error',signal:AbortSignal.timeout(15000)});if(!response.ok)throw Error('QUALITY_STATUS_UNAVAILABLE');
  const runs=(await response.json()).workflow_runs?.filter(r=>r.head_sha===env.GITHUB_SHA&&r.head_branch==='main'&&['push','workflow_dispatch'].includes(r.event));if(!runs?.length){if(attempt===89)throw Error('QUALITY_RUN_MISSING')}else{if(runs.some(r=>r.status==='completed'&&r.conclusion==='success'))return true;if(runs.every(r=>r.status==='completed'))throw Error('QUALITY_GATE_FAILED')}
  if(attempt%2===0)console.log('Waiting for validation of this exact commit.');await delay(30000);
 }throw Error('QUALITY_GATE_TIMEOUT');
}

if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){try{await requireQuality(process.env);console.log('Exact commit quality validated.')}catch(e){console.error(e.message);process.exitCode=1}}
