import {build} from 'esbuild';
import {mkdirSync,readdirSync,writeFileSync,unlinkSync} from 'node:fs';
import {resolve,basename} from 'node:path';
const here=import.meta.dirname;
const output=await build({absWorkingDir:here,entryPoints:['src/bridge.ts','src/worker.ts','src/host.ts','src/memory.ts','src/capabilities.ts','src/storage.ts','src/permissions.ts'],bundle:true,splitting:true,format:'esm',platform:'browser',target:'es2022',outdir:'dist',sourcemap:true,write:false});
const names=new Set(output.outputFiles.map(file=>basename(file.path)));
for(const folder of [resolve(here,'dist'),resolve(here,'../runtime/web/public/adapter')]){
 mkdirSync(folder,{recursive:true});
 // Only obsolete generated flat JS/map outputs are removed; no recursive filesystem cleanup.
 for(const file of readdirSync(folder,{withFileTypes:true}))if(file.isFile()&&/\.(js|map)$/.test(file.name)&&!names.has(file.name))unlinkSync(resolve(folder,file.name));
 for(const file of output.outputFiles)writeFileSync(resolve(folder,basename(file.path)),file.contents);
}
