// Local TEST ONLY: private KeyObjects are generated in memory and never exported.
import {generateKeyPairSync,sign,createHash} from 'node:crypto';
import {readFileSync,writeFileSync,mkdirSync,copyFileSync} from 'node:fs';
import {resolve,sep} from 'node:path';
const root=resolve(import.meta.dirname,'..'),options=new Map();
for(let i=2;i<process.argv.length;i+=2){const k=process.argv[i],v=process.argv[i+1];if(!['--archive','--manifest','--out'].includes(k)||!v||options.has(k))throw Error('Arguments: --archive file.tool --manifest manifest.json [--out target/directory]');options.set(k,v)}
if(!options.has('--archive')||!options.has('--manifest'))throw Error('Archive and manifest are required');
const out=resolve(root,options.get('--out')??'target/signing-test'),target=resolve(root,'target');if(!out.startsWith(target+sep))throw Error('Test output must stay under ignored target/');
const bytes=readFileSync(resolve(root,options.get('--archive')));if(bytes.length===0||bytes.length>25*1024*1024)throw Error('ARCHIVE_LIMIT');
const manifest=JSON.parse(readFileSync(resolve(root,options.get('--manifest')),'utf8'));
const rootKey=generateKeyPairSync('ed25519'),leaf=generateKeyPairSync('ed25519');const pub=k=>k.publicKey.export({format:'der',type:'spki'}).subarray(-32).toString('base64');
const anchor={id:'test-offline-root',public_key_base64:pub(rootKey)};
const keys={schema_version:1,version:1,issued_at:0,expires:4102444800000,root_key_id:anchor.id,keys:[{id:'test-release-key',public_key_base64:pub(leaf),not_before:0,expires:4102444800000,revoked:false}],signature:''};
const text=`kortex-keys-v1\n${keys.version}\n${keys.issued_at}\n${keys.expires}\n${keys.root_key_id}\n`+keys.keys.map(k=>`${k.id}|${k.public_key_base64}|${k.not_before}|${k.expires}|${k.revoked}\n`).join('');
keys.signature=sign(null,Buffer.from(text),rootKey.privateKey).toString('base64');
const descriptor={schema_version:1,tool_id:manifest.id,version:manifest.version,abi_version:manifest.abi_version,sha256:createHash('sha256').update(bytes).digest('hex'),signing_key_id:keys.keys[0].id,signature:''};
descriptor.signature=sign(null,Buffer.from(`kortex-artifact-v1\n${descriptor.tool_id}\n${descriptor.version}\n${descriptor.abi_version}\n${descriptor.sha256}\n`),leaf.privateKey).toString('base64');
mkdirSync(out,{recursive:true});for(const[name,value]of [['artifact-signed-test.json',descriptor],['trust-test-root.json',anchor],['trust-test-keys.json',keys]])writeFileSync(resolve(out,name),JSON.stringify(value,null,2)+'\n');copyFileSync(resolve(root,options.get('--archive')),resolve(out,'artifact-signed-test.tool'));
console.log(JSON.stringify({test_only:true,output:out,sha256:descriptor.sha256,private_keys_exported:false}));
