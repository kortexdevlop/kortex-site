import {releaseCli} from './release-cloud.mjs';
try{const action=process.env.KORTEX_RELEASE_ACTION;if(!['rollback','revoke'].includes(action))throw Error('INVALID_MAINTENANCE_ACTION');await releaseCli(action)}catch(e){console.error(e.message);process.exitCode=1}
