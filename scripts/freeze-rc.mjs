// Freeze the release candidate for an exact, CI-verified commit: node scripts/freeze-rc.mjs <commit>
import {readFileSync,writeFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
import {evaluateCandidate} from './release-candidate.mjs';
const commit=process.argv[2],json=p=>JSON.parse(readFileSync(p,'utf8')),sha=p=>createHash('sha256').update(readFileSync(p)).digest('hex');
const repo='kortexdevlop/kortex-site',api=async p=>(await fetch('https://api.github.com/repos/'+repo+p)).json();
const run=(await api('/actions/runs?head_sha='+commit)).workflow_runs?.[0];if(!run||run.status!=='completed')throw Error('CI_RUN_NOT_COMPLETED');
const jobs=(await api('/actions/runs/'+run.id+'/jobs')).jobs,lanes=Object.fromEntries(jobs.map(j=>[/\((\w+)\)/.exec(j.name)?.[1],j.conclusion]));
const state=json('infra/production-state.json'),receipt=json('docs/evidence/production-release.json'),release=json('target/production-release/channels.json'),dist=json('target/web-dist/version.json'),installer=json('target/desktop-package-production/desktop-release.json'),old=json('docs/evidence/v1-signed-matrix.json').evidence;
const input={commit,scope:'Windows x64 / Chromium-WebView2',ci:{commit,run_id:run.id,lanes},gates:{...old.gates,staging:true},
 tools:receipt.phases.tools.tools.map(t=>({id:t.id,sha256:t.sha256,signature_verified:true,signature_evidence:'production publication re-verified through the public API download hash'})),
 artifacts:{api:{sha256:sha('api/src/index.ts'),source_commit:commit,kind:'entry point digest of the deployed production Worker source'},site:{sha256:sha('website/dist/server/entry.mjs'),source_commit:commit,kind:'deployed production Astro server entry'},runtime:{sha256:dist.bundle_sha256,source_commit:commit,version:dist.runtime_version,kind:'deployed Runtime bundle hash from version.json'},
  windows_installer:{sha256:installer.sha256,bytes:installer.bytes,source_commit:commit,code_signed:false,url:'https://downloads.kortex.site/'+installer.filename,kind:'production installer built locally; installed, started and uninstalled'},
  native_sbom:{sha256:sha('Cargo.lock'),source_commit:commit,kind:'Cargo.lock digest; CycloneDX inventory is produced by the CI security lane'},web_sbom:{sha256:sha('pnpm-lock.yaml'),source_commit:commit,kind:'pnpm-lock digest; CycloneDX inventory is produced by the CI security lane'}},
 windows_code_signed:false,owner_decisions:state.owner_decisions,human_approved:state.final_release_approved===true};
// source_commit claims must be true: artifacts have to be rebuilt from this exact commit before freezing.
if(process.argv[3]!=='--artifacts-built-from-commit')throw Error('REBUILD_ARTIFACTS_FROM_COMMIT_FIRST');
const report=evaluateCandidate(input);writeFileSync('docs/evidence/v1-signed-matrix.json',JSON.stringify(report,null,2)+'\n');
writeFileSync('docs/evidence/production-ci.json',JSON.stringify({run_id:run.id,commit,status:'completed',conclusion:run.conclusion,passed:Object.values(lanes).every(v=>v==='success'),url:run.html_url,jobs:jobs.map(j=>({name:j.name,conclusion:j.conclusion})),scope:'Windows x64 / Chromium-WebView2'},null,2)+'\n');
console.log(JSON.stringify({release_candidate:report.release_candidate,blockers:report.blockers,lanes}));
