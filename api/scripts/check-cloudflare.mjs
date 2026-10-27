// Read-only identity/configuration check. Never starts login or prints credentials/account IDs.
import {spawnSync,spawn} from 'node:child_process';
import {readFileSync,mkdirSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
const root=resolve(import.meta.dirname,'../..');
const result=spawnSync(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),'whoami','--json','--config',resolve(root,'api/wrangler.jsonc')],{cwd:root,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'},encoding:'utf8',timeout:30000});
let identity;try{identity=JSON.parse(result.stdout)}catch{}
const explicitLoggedOut=/"loggedIn"\s*:\s*false/.test((result.stdout??'')+(result.stderr??''));
const authenticated=identity?.loggedIn===true?true:explicitLoggedOut?false:null;
const config=JSON.parse(readFileSync(resolve(root,'api/wrangler.jsonc'),'utf8'));
const report={date:new Date().toISOString().slice(0,10),checked_at:new Date().toISOString(),read_only:true,command:'wrangler whoami --json',exit_code:result.status,authenticated,
 verification:authenticated===true?'authenticated':authenticated===false?'not_authenticated':'verification_failed',
 auth_type:authenticated===true?identity.authType:null,account_count:authenticated===true&&Array.isArray(identity.accounts)?identity.accounts.length:0,
 token_permissions:authenticated===true?identity.tokenPermissions:undefined,
 credential_environment_present:Object.fromEntries(['CLOUDFLARE_API_TOKEN','CLOUDFLARE_API_KEY','CLOUDFLARE_EMAIL','CLOUDFLARE_ACCOUNT_ID'].map(name=>[name,Boolean(process.env[name])])),
 remote_configuration:['staging','production'].map(name=>({environment:name,account_configured:Boolean(config.account_id||config.env[name].account_id),d1_configured:Boolean(config.env[name].d1_databases?.length),r2_configured:Boolean(config.env[name].r2_buckets?.length),web_origin_configured:!config.env[name].vars.PUBLIC_WEB_ORIGIN.endsWith('.invalid')})),
 login_started:false,remote_writes:0};
if(process.argv.includes('--resources')&&authenticated===true){
 const commands=[['d1','list','--json'],['r2','bucket','list'],...['staging','production'].map(name=>['deployments','list','--name',`kortex-api-${name}`,'--json'])];
 report.resource_checks=await Promise.all(commands.map(args=>new Promise(resolveCheck=>{
  const child=spawn(process.execPath,[resolve(root,'node_modules/wrangler/bin/wrangler.js'),...args,'--config',resolve(root,'api/wrangler.jsonc')],{cwd:root,env:{...process.env,CI:'true',WRANGLER_SEND_METRICS:'false'},windowsHide:true});let stdout='',stderr='';
  child.stdout.on('data',data=>stdout+=data);child.stderr.on('data',data=>stderr+=data);
  const timeout=setTimeout(()=>child.kill(),30000);
  child.on('close',code=>{clearTimeout(timeout);const raw=stdout+stderr;let value;try{value=JSON.parse(stdout)}catch{}
   const rows=Array.isArray(value)?value:Array.isArray(value?.result)?value.result:undefined;
   resolveCheck({command:['wrangler',...args].join(' '),exit_code:code,
    status:code===0?'read_succeeded':/enable R2|R2.*not enabled|R2.*enabled.*account/i.test(raw)?'r2_activation_required':/not found|does not exist|could not be found|10007/i.test(raw)?'not_found':/authentication|permission|unauthorized|forbidden/i.test(raw)?'access_denied':'verification_failed',
    resource_count:rows?.length,kortex_resources:rows?.filter(x=>typeof x.name==='string'&&/kortex/i.test(x.name)).map(x=>x.name),
    r2_kortex_name_lines:args[0]==='r2'&&code===0?stdout.split(/\r?\n/).filter(line=>/^name:\s*kortex/i.test(line)).map(line=>line.trim()):undefined,
    error_codes:[...raw.matchAll(/\[code:\s*(\d+)\]/g)].map(m=>m[1])});
  });child.on('error',()=>{clearTimeout(timeout);resolveCheck({command:['wrangler',...args].join(' '),exit_code:null,status:'verification_failed'})});
 })));
}
report.remote_ready=authenticated===true&&report.remote_configuration.every(env=>env.account_configured&&env.d1_configured&&env.r2_configured&&env.web_origin_configured)&&Boolean(report.resource_checks?.every(check=>check.status==='read_succeeded'));
mkdirSync(resolve(root,'target'),{recursive:true});writeFileSync(resolve(root,'target/cloudflare-connection-report.json'),JSON.stringify(report,null,2)+'\n');console.log(JSON.stringify(report,null,2));
if(authenticated===null)process.exitCode=1;
