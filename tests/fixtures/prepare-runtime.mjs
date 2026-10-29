import {createPrivateKey,createPublicKey,sign} from 'node:crypto';
import {writeFileSync} from 'node:fs';
import {runtimeSigningText,certificateText} from '../../shared/typescript/src/runtime.ts';
const key=n=>createPrivateKey({key:Buffer.concat([Buffer.from('302e020100300506032b657004220420','hex'),Buffer.alloc(32,n)]),format:'der',type:'pkcs8'}),rootKey=key(81),releaseKey=key(80),pub=k=>createPublicKey(k).export({format:'der',type:'spki'}).subarray(-32).toString('base64');
const now=1700000000000,root={id:'runtime-test-root',public_key_base64:pub(rootKey)},keys={schema_version:1,version:1,issued_at:now-1000,expires:now+86400000,root_key_id:root.id,keys:[{id:'runtime-test-release',public_key_base64:pub(releaseKey),not_before:now-1000,expires:now+86400000,revoked:false}],signature:''};
keys.signature=sign(null,certificateText(keys),rootKey).toString('base64');
const document={schema_version:1,sequence:7,issued_at:now,expires:now+3600000,channels:{stable:null,staging:{runtime_version:'1.0.1',abi_versions:['1.0'],protocol_versions:[1],ui_schema_versions:['1.0'],web:{origin:'https://run.example.test',version_url:'https://run.example.test/version.json',bundle_sha256:'1'.repeat(64),script_integrity:'sha256-'+Buffer.alloc(32,1).toString('base64')},desktop:null}},signing_key_id:keys.keys[0].id,signature:''};
document.signature=sign(null,runtimeSigningText(document),releaseKey).toString('base64');
writeFileSync('tests/fixtures/runtime-channels.signed-test.json',JSON.stringify({now,root,envelope:{document,keys},signing_text:Buffer.from(runtimeSigningText(document)).toString('utf8')},null,2)+'\n');
