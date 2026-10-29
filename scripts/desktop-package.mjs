import {homedir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {mkdirSync,readdirSync,readFileSync,writeFileSync,copyFileSync,existsSync} from 'node:fs';
import {resolve,join} from 'node:path';
import {createHash} from 'node:crypto';
import {pathToFileURL} from 'node:url';
export function packageDesktop({channel='staging',out=resolve('target/desktop-package'),run=spawnSync}={}){
 if(process.platform!=='win32')throw Error('OS excluded by owner: Windows only');
 if(channel!=='staging')throw Error('Production requires an operator-provided Windows signing certificate');
 mkdirSync(out,{recursive:true});
 const result=run('dx',['bundle','--desktop','--release','--package','kortex-desktop','--package-types','nsis','--out-dir',out,'--locked'],{cwd:resolve('desktop'),encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0'}});
 writeFileSync(join(out,'build.log'),(result.stdout||'')+(result.stderr||''));
 if(result.status!==0){
  const log=(result.stdout||'')+(result.stderr||'');
  if(!log.includes('makensis failed')||!log.includes('(ACP)')||!log.includes('no files found'))throw Error('Desktop installer build failed; see build.log');
  const script=resolve('target/dx/kortex-desktop/bundle/windows/nsis/KortexDesktop.nsi'),compiler=resolve(homedir(),'.dx/tools/nsis-3.11/makensis.exe');if(!existsSync(script)||!existsSync(compiler))throw Error('NSIS UTF-8 retry unavailable');
  const retry=run(compiler,['/INPUTCHARSET','UTF8','/OUTPUTCHARSET','UTF8',script],{encoding:'utf8',windowsHide:true,maxBuffer:16*1024*1024});writeFileSync(join(out,'nsis-utf8.log'),(retry.stdout||'')+(retry.stderr||''));if(retry.status!==0)throw Error('NSIS UTF-8 packaging failed');
  const produced=resolve('target/dx/kortex-desktop/bundle/windows/nsis/KortexDesktop_'+/^version\s*=\s*"([^"]+)"/m.exec(readFileSync('desktop/Cargo.toml','utf8'))[1]+'_x64-setup.exe');copyFileSync(produced,join(out,produced.split(/[\\/]/).pop()));
 }

 const files=readdirSync(out).filter(f=>/\.exe$/i.test(f));if(files.length!==1)throw Error('Exactly one NSIS installer expected');
 const file=join(out,files[0]),bytes=readFileSync(file);if(bytes[0]!==0x4d||bytes[1]!==0x5a)throw Error('Invalid Windows executable');
 const version=/^version\s*=\s*"([^"]+)"/m.exec(readFileSync('desktop/Cargo.toml','utf8'))?.[1];if(!version)throw Error('Missing Desktop version');
 const descriptor={schema_version:1,platform:'windows-x64',channel,version,format:'nsis',filename:files[0],sha256:createHash('sha256').update(bytes).digest('hex'),bytes:bytes.length,code_signed:false,production_ready:false};
 writeFileSync(join(out,'desktop-release.json'),JSON.stringify(descriptor,null,2)+'\n');return descriptor;
}
if(process.argv[1]&&import.meta.url===pathToFileURL(resolve(process.argv[1])).href){try{console.log(JSON.stringify(packageDesktop()))}catch(e){console.error(e.message);process.exitCode=1}}
