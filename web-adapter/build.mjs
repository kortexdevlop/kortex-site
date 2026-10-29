import './generate-validators.mjs';
import {build} from 'esbuild';
import {mkdirSync,readdirSync,writeFileSync,unlinkSync} from 'node:fs';
import {resolve,basename} from 'node:path';
const here=import.meta.dirname;
const output=await build({absWorkingDir:here,entryPoints:['src/ai-bridge.ts','src/bridge.ts','src/worker.ts','src/host.ts','src/memory.ts','src/capabilities.ts','src/storage.ts','src/permissions.ts','src/trust.ts','src/archive.ts','src/loader.ts','src/iframe.ts','src/interop.ts','src/revocation.ts','src/runtime-channels.ts'],plugins:[{name:'csp-contracts',setup(build){build.onResolve({filter:/^@kortex\/contracts$/},()=>({path:resolve(here,'src/browser-contracts.ts')}))}}],bundle:true,splitting:true,format:'esm',platform:'browser',target:'es2022',outdir:'dist',sourcemap:true,write:false});
const names=new Set(output.outputFiles.map(file=>basename(file.path)));
for(const folder of [resolve(here,'dist'),resolve(here,'../runtime/web/public/adapter')]){
 mkdirSync(folder,{recursive:true});
 // Only obsolete generated flat JS/map outputs are removed; no recursive filesystem cleanup.
 for(const file of readdirSync(folder,{withFileTypes:true}))if(file.isFile()&&/\.(js|map)$/.test(file.name)&&!names.has(file.name))unlinkSync(resolve(folder,file.name));
 for(const file of output.outputFiles)writeFileSync(resolve(folder,basename(file.path)),file.contents);
}


writeFileSync(resolve(here,'../runtime/web/public/tool-shell.css'),(await import('node:fs')).readFileSync(resolve(here,'../runtime/assets/tool-shell.css')));
writeFileSync(resolve(here,'../runtime/web/public/runtime-style.css'),(await import('node:fs')).readFileSync(resolve(here,'../runtime/web/assets/style.css')));
