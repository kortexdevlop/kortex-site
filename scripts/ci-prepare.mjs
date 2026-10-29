// Reproducible, local-only fixtures. No production credentials or remote publication.
import {spawnSync} from 'node:child_process';import {existsSync,mkdirSync,readFileSync,writeFileSync,cpSync} from 'node:fs';import {resolve} from 'node:path';import {prepareRelease} from './release-tool.mjs';
const root=process.cwd(),fixtures=resolve('target/ci-fixtures');mkdirSync(fixtures,{recursive:true});const run=(bin,args,cwd=root,extra={})=>{const r=spawnSync(bin,args,{cwd,stdio:'inherit',windowsHide:true,env:{...process.env,...extra}});if(r.status!==0)throw Error('Fixture preparation failed: '+bin+' '+args[0])};
run('cargo',['fetch','--locked']);
run(process.execPath,['api/scripts/generate-validators.mjs']);
run('cargo',['build','--manifest-path','tests/spikes/core-guest/Cargo.toml','--target','wasm32-unknown-unknown','--release','--locked'],root,{RUSTFLAGS:'-C link-arg=--max-memory=67108864'});
run('cargo',['build','--manifest-path','tests/fixtures/workflow-capability-guest/Cargo.toml','--target','wasm32-unknown-unknown','--release','--locked'],root,{RUSTFLAGS:'-C link-arg=--max-memory=67108864',CARGO_TARGET_DIR:resolve('target/workflow-capability-guest')});
run(process.execPath,['web-adapter/prepare-tests.mjs']);run(process.execPath,['web-adapter/build.mjs']);
const reports=[];for(const tool of ['calculatrice','convertisseur-casse','calculateur-hash','timestamp-utc','explorateur-csv'])reports.push(prepareRelease({tool,out:resolve(fixtures,tool),dryRun:true}));writeFileSync(resolve(fixtures,'releases.json'),JSON.stringify(reports,null,2)+'\n');
if(!process.argv.includes('--native')&&!existsSync('target/web-dist/index.html'))run(process.execPath,['scripts/build-runtime.mjs']);
if(!process.argv.includes('--native'))cpSync('web-adapter/dist','target/web-dist/adapter',{recursive:true});console.log('Signed test fixtures prepared locally; no remote writes.');
