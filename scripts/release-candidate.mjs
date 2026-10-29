import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {pathToFileURL} from 'node:url';
export function evaluateCandidate(input){
 const blockers=[];
 if(!/^[a-f0-9]{40}$/.test(input.commit??''))blockers.push('source_commit');
 const lanes=['contracts','native','web','security'];
 if(input.ci?.commit!==input.commit)blockers.push('ci_same_commit');
 for(const lane of lanes)if(input.ci?.lanes?.[lane]!=='success')blockers.push('ci_'+lane);
 for(const gate of ['staging','runtime_performance','design_a11y','resilience','security'])if(input.gates?.[gate]!==true)blockers.push(gate);
 if(input.scope!=='Windows x64 / Chromium-WebView2')blockers.push('scope');
 if(!Array.isArray(input.tools)||input.tools.length!==5||new Set(input.tools?.map(t=>t.id)).size!==5||input.tools.some(t=>!/^[a-f0-9]{64}$/.test(t.sha256??'')||t.signature_verified!==true))blockers.push('five_signed_tools');
 for(const name of ['api','site','runtime','windows_installer','native_sbom','web_sbom'])if(!/^[a-f0-9]{64}$/.test(input.artifacts?.[name]?.sha256??''))blockers.push('artifact_'+name);
 if(input.windows_code_signed!==true)blockers.push('windows_code_signing');
 if(input.human_approved!==true)blockers.push('human_approval');
 return {schema_version:1,date:new Date().toISOString(),commit:input.commit??null,scope:input.scope??null,release_candidate:blockers.length===0,production_ready:false,blockers,evidence:input};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href){
 const input=JSON.parse(readFileSync(process.argv[2]||'target/ci-reports/candidate-input.json','utf8')),out=process.argv[3]||'target/ci-reports/release-candidate.json';
 mkdirSync(new URL('.',pathToFileURL(out)),{recursive:true});const report=evaluateCandidate(input);writeFileSync(out,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({release_candidate:report.release_candidate,blockers:report.blockers}));if(!report.release_candidate)process.exitCode=2;
}
