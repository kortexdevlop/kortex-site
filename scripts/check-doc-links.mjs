import {readFileSync,existsSync,statSync,mkdirSync,writeFileSync} from 'node:fs';
import {resolve,dirname,relative,sep} from 'node:path';import {execFileSync} from 'node:child_process';import {pathToFileURL} from 'node:url';
const strip=text=>text.replace(/^(`{3,}|~{3,})[^\n]*\n[\s\S]*?^\1\s*$/gm,'').replace(/`[^`\n]*`/g,'');
export function anchors(text){const used=new Map(),out=new Set();for(const m of text.matchAll(/^#{1,6}\s+(.+?)\s*#*\s*$/gm)){const base=m[1].replace(/<[^>]*>/g,'').toLowerCase().replace(/[^\p{L}\p{N}_\-\s]/gu,'').trim().replace(/\s/g,'-'),n=used.get(base)??0;out.add(base+(n?'-'+n:''));used.set(base,n+1)}for(const m of text.matchAll(/\bid=["']([^"']+)["']/g))out.add(m[1]);return out}
export function checkFile(root,file){const text=readFileSync(resolve(root,file),'utf8'),clean=strip(text),issues=[],links=[];const refs=new Map([...clean.matchAll(/^\s*\[([^\]]+)\]:\s*<?([^\s>]+)>?/gm)].map(m=>[m[1].toLowerCase(),m[2]]));
 for(const m of clean.matchAll(/!?\[[^\]\n]*\]\((<[^>]+>|[^\s)]+)(?:\s+["'][^\n]*?["'])?\)|!?\[([^\]\n]+)\]\[([^\]\n]*)\]/g)){const target=m[1]?.replace(/^<|>$/g,'')??refs.get((m[3]||m[2]).toLowerCase());if(!target)continue;/* An undefined reference is plain text in Markdown (also used by regexes). */if(/^(?:https?:|mailto:)/i.test(target))continue;
  let decoded;try{decoded=decodeURIComponent(target)}catch{issues.push({file,error:'invalid-encoding',link:target});continue}const [name,fragment]=decoded.split('#'),absolute=resolve(dirname(resolve(root,file)),name||'.');const path=name?absolute:resolve(root,file);links.push(target);
  if(/^[A-Za-z][A-Za-z0-9+.-]*:/.test(name)||!path.startsWith(resolve(root)+sep)){issues.push({file,error:'nonportable-or-outside-repository',link:target});continue}
  if(!existsSync(path)){issues.push({file,error:'missing-target',link:target});continue}
  if(fragment&&statSync(path).isFile()){if(/\.md$/i.test(path)){if(!anchors(readFileSync(path,'utf8')).has(fragment))issues.push({file,error:'missing-anchor',link:target})}else if(/^L\d+(?:-L\d+)?$/.test(fragment)){const line=Number(fragment.match(/\d+/)[0]);if(line>readFileSync(path,'utf8').split('\n').length)issues.push({file,error:'missing-source-line',link:target})}}
 }return{links,issues};
}
export function audit(root=process.cwd()){
 const files=execFileSync('git',['ls-files','--cached','--others','--exclude-standard'],{cwd:root,encoding:'utf8'}).trim().split(/\r?\n/).filter(Boolean),steps=new Set([...readFileSync(resolve(root,'docs/MASTER_PLAN.md'),'utf8').matchAll(/^### Étape (\d+\.\d+)\b/gm)].map(m=>m[1]));
 const ownership=JSON.parse(readFileSync(resolve(root,'docs/FILE_OWNERS.json'),'utf8')),issues=[],owners=[];let links=0,documents=0;
 for(const file of files){if(!existsSync(resolve(root,file)))continue;if(file.endsWith('.md')){documents++;const result=checkFile(root,file);links+=result.links.length;issues.push(...result.issues)}
  const explicit=/^docs\/reports\/STEP-(\d+\.\d+)\.md$/.exec(file),rule=explicit?{step:explicit[1],role:'Responsable du rapport de cette étape'}:ownership.rules.find(r=>r.files?.includes(file)||r.prefix&&file.startsWith(r.prefix));if(!rule||!steps.has(rule.step)||!rule.role)issues.push({file,error:'missing-step-owner'});else owners.push({file,step:rule.step,role:rule.role});
 }return{date:new Date().toISOString(),documents,internal_links:links,files_owned:owners.length,issues,owners,human_approval:false};
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){const report=audit(),out=process.argv[process.argv.indexOf('--report')+1]||'target/ci-reports/documentation.json';const path=process.argv.includes('--report')?out:'target/ci-reports/documentation.json';mkdirSync(dirname(resolve(path)),{recursive:true});writeFileSync(path,JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify({documents:report.documents,internal_links:report.internal_links,files_owned:report.files_owned,issues:report.issues}));if(report.issues.length)process.exitCode=1}
