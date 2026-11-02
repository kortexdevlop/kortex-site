// Windows, once: copy the active production release seed out of DPAPI into a passphrase-sealed file
// (~/.kortex-offline/<slot>.sealed) that `pnpm tool:publish` can open on a Mac or Linux machine.
// Copy that file to the same folder on the other machine; never commit it.
import {readFileSync,writeFileSync,existsSync} from 'node:fs';
import {resolve} from 'node:path';
import {homedir} from 'node:os';
import {spawnSync} from 'node:child_process';
import {sealSeed,openSeed,askPassphrase} from './portable-key.mjs';

if(process.platform!=='win32')throw Error('WINDOWS_ONLY: la clé DPAPI ne se lit que sur le PC d’origine');
const root=resolve(import.meta.dirname,'..'),keys=JSON.parse(readFileSync(resolve(root,'infra/trust/release-production-keys.json'),'utf8'));
const activeKey=keys.keys.filter(k=>!k.revoked).at(-1),slot=activeKey.id==='production-release-1'?'release-production':'release-production-'+activeKey.id.split('-').pop();
const folder=resolve(homedir(),'.kortex-offline'),source=resolve(folder,slot+'.dpapi'),target=resolve(folder,slot+'.sealed');
if(existsSync(target))throw Error('ALREADY_EXPORTED '+target+' (supprimer le fichier pour le refaire)');
const r=spawnSync('powershell.exe',['-NoProfile','-NonInteractive','-Command','Add-Type -AssemblyName System.Security; [Convert]::ToBase64String([Security.Cryptography.ProtectedData]::Unprotect([Convert]::FromBase64String([Console]::In.ReadToEnd().Trim()),$null,[Security.Cryptography.DataProtectionScope]::CurrentUser))'],{input:readFileSync(source,'utf8'),encoding:'utf8',windowsHide:true});
if(r.status!==0)throw Error('DPAPI_UNAVAILABLE');
const seed=Buffer.from(r.stdout.trim(),'base64');
try{
 if(seed.length!==32)throw Error('UNEXPECTED_SEED_LENGTH');
 const first=await askPassphrase('Passphrase (12 caractères min.) : '),again=await askPassphrase('Confirmer : ');
 if(first!==again)throw Error('PASSPHRASES_DIFFER');
 const sealed=sealSeed(seed,first),check=openSeed(sealed,first);
 try{if(!check.equals(seed))throw Error('SEAL_ROUNDTRIP_FAILED')}finally{check.fill(0)}
 writeFileSync(target,sealed,{flag:'wx'});
 console.log('Clé '+activeKey.id+' exportée : '+target+'\nCopie ce fichier dans ~/.kortex-offline/ sur le Mac. Ne le commite jamais.');
}finally{seed.fill(0)}
