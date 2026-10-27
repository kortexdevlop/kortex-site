import {createServer} from 'node:http';
import {readFileSync,writeFileSync,mkdirSync} from 'node:fs';
import {resolve,extname,sep} from 'node:path';
import {spawn,spawnSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {randomBytes} from 'node:crypto';
import {root,wrangler,toolSQL,executeSQL} from './fixtures.mjs';
const dir=resolve(root,'target/website-integration');mkdirSync(dir,{recursive:true});
// A fresh SQLite directory per run avoids mutating any developer/staging database.
const state=resolve(dir,'state-'+Date.now());
const api={name:'kortex-api-local',main:resolve(root,'api/src/index.ts'),compatibility_date:'2026-10-26',vars:{KORTEX_ENV:'local',PUBLIC_WEB_ORIGIN:'http://127.0.0.1:4321',CURSOR_SIGNING_KEY:randomBytes(32).toString('hex')},d1_databases:[{binding:'DB',database_name:'website-integration',database_id:'00000000-0000-4000-8000-000000000093',migrations_dir:resolve(root,'database/migrations')}],r2_buckets:[{binding:'TOOLS_BUCKET',bucket_name:'website-integration'}],send_metrics:false};
writeFileSync(resolve(dir,'api.json'),JSON.stringify(api));
// Store the selected local state for the D1 mutation tests.
writeFileSync(resolve(dir,'state-path.txt'),state);
const require=createRequire(import.meta.url),{makeTool}=require('../fixtures/make-tool.cjs');
const fixture=makeTool({wasmPath:resolve(root,'target/calculatrice-fixture/module.wasm'),manifestPath:resolve(root,'tools/calculatrice/manifest.json'),contractPath:resolve(root,'tools/calculatrice/contract.json'),iconPath:resolve(root,'tools/calculatrice/assets/icon.svg')});
writeFileSync(resolve(dir,'fixture.json'),JSON.stringify({descriptor:fixture.descriptor,keys:fixture.keys}));
const migrate=spawnSync(process.execPath,[wrangler,'d1','migrations','apply','DB','--local','--config',resolve(dir,'api.json'),'--persist-to',state],{cwd:root,encoding:'utf8'});if(migrate.status!==0)throw Error(migrate.stdout+migrate.stderr);
executeSQL(`INSERT INTO signing_keys VALUES('${fixture.descriptor.signing_key_id}','${fixture.keys[0].public_key_base64}',1,0,9999999999,0);\n`+toolSQL('calculatrice','Calculatrice')+toolSQL('outil-cache','Outil caché',{published:false})+Array.from({length:5},(_,i)=>toolSQL('photo-'+i,'Photo '+i,{category:'image'})).join('')+Array.from({length:30},(_,i)=>toolSQL('entry-'+String(i).padStart(2,'0'),'Entrée '+i,{category:'developer'})).join(''));
const children=[];
function launch(args,cwd=root){const child=spawn(process.execPath,[wrangler,...args],{cwd,stdio:['ignore','pipe','pipe'],windowsHide:true,env:{...process.env,WRANGLER_SEND_METRICS:'false'}});children.push(child);child.stdout.on('data',data=>{if(/ERROR/.test(String(data)))process.stderr.write(data)});child.stderr.on('data',data=>process.stderr.write(data));return child;}
const runtime=createServer((req,res)=>{try{const path=new URL(req.url,'http://local').pathname;res.setHeader('Content-Security-Policy',"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'; img-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors http://127.0.0.1:4321");res.setHeader('X-Content-Type-Options','nosniff');
 if(path==='/embed-config.js'){res.setHeader('Content-Type','text/javascript');res.end('window.kortexEmbedConfig='+JSON.stringify({parent_origin:'http://127.0.0.1:4321',api_origin:'http://127.0.0.1:4340',keys:fixture.keys,artifacts:[{...fixture.descriptor,url:'http://127.0.0.1:4340/calculatrice.tool'}]})+';');return;}
 if(path==='/calculatrice.tool'){res.setHeader('Content-Type','application/zip');res.end(fixture.bytes);return;}
 const base=resolve(root,'target/web-dist'),file=path==='/frame.html'?resolve(root,'runtime/web/public/frame.html'):resolve(base,'.'+path);if(path!=='/frame.html'&&!file.startsWith(base+sep))throw Error('path');res.setHeader('Content-Type',({'.html':'text/html','.js':'text/javascript','.wasm':'application/wasm','.css':'text/css'})[extname(file)]||'application/octet-stream');res.end(readFileSync(file));
 }catch{res.statusCode=404;res.end();}});
await new Promise((done,reject)=>{runtime.once('error',reject);runtime.listen(4340,'127.0.0.1',done);});

const compiled=resolve(root,'website/dist/server');

const original=JSON.parse(readFileSync(resolve(compiled,'wrangler.json')));
const site={...original,name:'kortex-web-integration',topLevelName:'kortex-web-integration',configPath:resolve(dir,'site.json'),userConfigPath:resolve(dir,'site.json'),main:resolve(compiled,original.main),assets:{...original.assets,directory:resolve(compiled,original.assets.directory)},vars:{WEB_ORIGIN:'http://127.0.0.1:4321',PUBLIC_RUN_ORIGIN:'http://127.0.0.1:4340'}};
writeFileSync(resolve(dir,'site.json'),JSON.stringify(site));
launch(['dev','--config',resolve(dir,'site.json'),'--config',resolve(dir,'api.json'),'--local','--persist-to',state,'--ip','127.0.0.1','--port','4321']);
const failed={...original,topLevelName:'kortex-web-unavailable-test',configPath:resolve(dir,'unavailable.json'),userConfigPath:resolve(dir,'unavailable.json'),name:'kortex-web-unavailable-test',main:resolve(compiled,original.main),services:[],assets:{...original.assets,directory:resolve(compiled,original.assets.directory)},vars:{WEB_ORIGIN:'http://127.0.0.1:4331'}};
writeFileSync(resolve(dir,'unavailable.json'),JSON.stringify(failed));
launch(['dev','--config',resolve(dir,'unavailable.json'),'--local','--ip','127.0.0.1','--port','4331']);
function stop(){for(const child of children)child.kill();runtime.closeAllConnections();runtime.close();setTimeout(()=>process.exit(),500).unref();}
process.on('SIGINT',stop);process.on('SIGTERM',stop);
console.log('Website integration: local D1, compiled SSR and signed WASM runtime starting.');
