import {build} from 'esbuild';
await build({entryPoints:['src/bridge.ts','src/worker.ts','src/host.ts','src/memory.ts'],bundle:true,splitting:true,format:'esm',platform:'browser',target:'es2022',outdir:'dist',sourcemap:true});


