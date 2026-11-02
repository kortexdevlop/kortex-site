// Passphrase-sealed copy of an operator signing seed, readable on any OS (Mac, Linux, Windows).
// Format: JSON {v,kdf,N,r,p,salt,iv,tag,ct}, scrypt + AES-256-GCM. The passphrase never touches disk.
import {scryptSync,randomBytes,createCipheriv,createDecipheriv} from 'node:crypto';
import {createInterface} from 'node:readline';

const N=1<<17,R=8,P=1,maxmem=256*1024*1024;
const derive=(passphrase,salt,s)=>scryptSync(Buffer.from(passphrase,'utf8'),salt,32,{N:s.N,r:s.r,p:s.p,maxmem});

export function sealSeed(seed,passphrase){
 if(passphrase.length<12)throw Error('PASSPHRASE_TOO_SHORT (12 caractères minimum)');
 const salt=randomBytes(16),iv=randomBytes(12),key=derive(passphrase,salt,{N,r:R,p:P});
 try{const c=createCipheriv('aes-256-gcm',key,iv),ct=Buffer.concat([c.update(seed),c.final()]);
  return JSON.stringify({v:1,kdf:'scrypt',N,r:R,p:P,salt:salt.toString('base64'),iv:iv.toString('base64'),tag:c.getAuthTag().toString('base64'),ct:ct.toString('base64')})+'\n'}
 finally{key.fill(0)}
}

export function openSeed(text,passphrase){
 const s=JSON.parse(text);if(s.v!==1||s.kdf!=='scrypt')throw Error('SEALED_KEY_FORMAT_UNKNOWN');
 const key=derive(passphrase,Buffer.from(s.salt,'base64'),s);
 try{const d=createDecipheriv('aes-256-gcm',key,Buffer.from(s.iv,'base64'));d.setAuthTag(Buffer.from(s.tag,'base64'));
  try{return Buffer.concat([d.update(Buffer.from(s.ct,'base64')),d.final()])}catch{throw Error('WRONG_PASSPHRASE')}}
 finally{key.fill(0)}
}

// KORTEX_RELEASE_PASSPHRASE wins (scripts), otherwise a hidden prompt on the terminal.
export async function askPassphrase(question){
 if(process.env.KORTEX_RELEASE_PASSPHRASE)return process.env.KORTEX_RELEASE_PASSPHRASE;
 if(!process.stdin.isTTY)throw Error('PASSPHRASE_REQUIRED (terminal interactif ou KORTEX_RELEASE_PASSPHRASE)');
 const rl=createInterface({input:process.stdin,output:process.stdout,terminal:true});let muted=false;
 rl._writeToOutput=s=>{if(!muted||s.includes('\n'))process.stdout.write(muted?'\n':s)};
 try{return await new Promise(done=>{rl.question(question,a=>done(a));muted=true})}finally{rl.close()}
}
