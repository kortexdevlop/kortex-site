import {spawnSync} from 'node:child_process';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'..');
const result=spawnSync('cargo',['build','--manifest-path',resolve(root,'tests/fixtures/capability-guest/Cargo.toml'),'--target','wasm32-unknown-unknown','--release','--locked'],{cwd:root,stdio:'inherit',env:{...process.env,CARGO_TARGET_DIR:resolve(root,'target/web-test-guest'),RUSTFLAGS:'-C link-arg=--max-memory=67108864'}});
if(result.status!==0)process.exit(result.status??1);

const calculator=spawnSync(process.execPath,[resolve(root,'tests/fixtures/prepare-calculatrice.mjs')],{cwd:root,stdio:'inherit',env:process.env});
if(calculator.status!==0)process.exit(calculator.status??1);

// Generate the same correctly signed hostile archives exercised by native admission.
const security=spawnSync('cargo',['test','-p','kortex-runtime-native','--test','package_security','--locked'],{cwd:root,stdio:'inherit',env:process.env});
if(security.status!==0)process.exit(security.status??1);
