import {appendFileSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
import {resolve} from 'node:path';
import {channels} from './ci-publish-tool.mjs';
import {requireQuality} from './ci-quality.mjs';

export function assertReusableDesktop(run,jobs,comparison){
 if(String(run.head_repository?.id)!==channels.repository_id||run.head_branch!=='main'||!['push','workflow_dispatch'].includes(run.event)||run.path!=='.github/workflows/desktop.yml'||run.status!=='completed'||!/^[a-f0-9]{40}$/.test(run.head_sha||''))throw Error('UNTRUSTED_DESKTOP_ARTIFACT_SOURCE');
 if(!jobs.some(j=>j.name==='windows'&&j.steps?.some(s=>['Run node scripts/desktop-package.mjs','Build Windows installer'].includes(s.name)&&s.conclusion==='success')))throw Error('DESKTOP_BUILD_NOT_PROVEN');
 if(!['ahead','identical'].includes(comparison.status)||!Array.isArray(comparison.files)||comparison.files.length>=300||comparison.files.some(f=>/^(desktop\/|runtime\/desktop\/|crates\/|shared\/|infra\/(trust|desktop)\/|Cargo\.(toml|lock)$|rust-toolchain\.toml$|scripts\/desktop-(package|webview2)\.mjs$)/.test(f.filename)))throw Error('DESKTOP_REBUILD_REQUIRED');
 return run.head_sha;
}
export async function desktopArtifactSource(env,runId,fetcher=fetch){
 if(env.GITHUB_REPOSITORY!==channels.repository||env.GITHUB_REPOSITORY_ID!==channels.repository_id||env.GITHUB_REF!==channels.branch||!/^[a-f0-9]{40}$/.test(env.GITHUB_SHA||''))throw Error('UNTRUSTED_DESKTOP_CONTEXT');
 if(!runId)return env.GITHUB_SHA;
 if(!/^[1-9][0-9]{0,15}$/.test(runId))throw Error('INVALID_DESKTOP_RUN');
 const get=async path=>{const r=await fetcher('https://api.github.com/repos/'+channels.repository+path,{headers:{Authorization:'Bearer '+env.GITHUB_TOKEN,Accept:'application/vnd.github+json'},redirect:'error',signal:AbortSignal.timeout(15000)});if(!r.ok)throw Error('DESKTOP_SOURCE_UNAVAILABLE');return r.json()};
 const run=await get('/actions/runs/'+runId),jobs=await get('/actions/runs/'+runId+'/jobs?filter=all&per_page=100'),comparison=await get('/compare/'+run.head_sha+'...'+env.GITHUB_SHA);
 const source=assertReusableDesktop(run,jobs.jobs??[],comparison);await requireQuality({...env,GITHUB_SHA:source},fetcher);return source;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){try{const runId=process.env.KORTEX_DESKTOP_ARTIFACT_RUN||'';const sha=await desktopArtifactSource(process.env,runId);if(process.env.GITHUB_OUTPUT)appendFileSync(process.env.GITHUB_OUTPUT,'sha='+sha+'\n');console.log(JSON.stringify({source_commit:sha,reused:Boolean(runId)}))}catch(e){console.error(e.message);process.exitCode=1}}
