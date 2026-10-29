import {spawnSync} from 'node:child_process';
import {mkdirSync,readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {cpus,totalmem,release} from 'node:os';
if(process.platform!=='win32')throw Error('WINDOWS_ONLY_REFERENCE_SCOPE');
const report=resolve(process.env.KORTEX_NATIVE_PERF_REPORT||'target/ci-reports/runtime-native-performance.json');mkdirSync(resolve(report,'..'),{recursive:true});
const result=spawnSync('cargo',['test','-p','kortex-runtime-native','--test','runtime_performance','--features','performance-measurements','--locked','--profile','desktop-release','--target','x86_64-pc-windows-msvc','--config','profile.desktop-release.inherits="release"','--config','profile.desktop-release.strip=false','--','--nocapture'],{stdio:'inherit',windowsHide:true,env:{...process.env,CARGO_INCREMENTAL:'0',KORTEX_NATIVE_PERF_REPORT:report,KORTEX_PERF_ENFORCE:'1'}});
if(result.status!==0)process.exit(result.status??1);
const data=JSON.parse(readFileSync(report));data.hardware={cpu:cpus()[0].model,logical_cpus:cpus().length,memory_bytes:totalmem(),windows:release()};writeFileSync(report,JSON.stringify(data,null,2)+'\n');
