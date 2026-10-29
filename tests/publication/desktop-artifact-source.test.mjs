import {test} from 'node:test';
import assert from 'node:assert/strict';
import {assertReusableDesktop} from '../../scripts/desktop-artifact-source.mjs';
const run={head_repository:{id:1437766566},head_branch:'main',event:'workflow_dispatch',path:'.github/workflows/desktop.yml',status:'completed',head_sha:'a'.repeat(40)},jobs=[{name:'windows',steps:[{name:'Run node scripts/desktop-package.mjs',conclusion:'success'}]}],diff={status:'ahead',files:[{filename:'scripts/desktop-installer-smoke.ps1'},{filename:'.github/workflows/desktop.yml'}]};
test('reuses a proven immutable installer for a smoke-only fix with its original source',()=>{assert.equal(assertReusableDesktop(run,jobs,diff),run.head_sha)});
test('rejects foreign sources, unproven builds and changes that require a new installer',()=>{
 for(const change of [{head_repository:{id:1}},{event:'pull_request'},{head_branch:'other'},{path:'.github/workflows/other.yml'},{status:'in_progress'}])assert.throws(()=>assertReusableDesktop({...run,...change},jobs,diff),/UNTRUSTED/);
 assert.throws(()=>assertReusableDesktop(run,[],diff),/BUILD_NOT_PROVEN/);
 for(const filename of ['desktop/src/main.rs','crates/kortex-ui/src/lib.rs','shared/rust/src/lib.rs','infra/trust/release-staging-root.json','Cargo.lock','scripts/desktop-package.mjs'])assert.throws(()=>assertReusableDesktop(run,jobs,{status:'ahead',files:[{filename}]}),/REBUILD_REQUIRED/);
 assert.throws(()=>assertReusableDesktop(run,jobs,{status:'behind',files:[]}),/REBUILD_REQUIRED/);
});
