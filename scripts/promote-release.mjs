import {releaseCli} from './release-cloud.mjs';
try{await releaseCli('promote')}catch(e){console.error(e.message);process.exitCode=1}
