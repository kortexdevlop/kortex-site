import { test, expect, chromium, webkit } from 'playwright/test';
import { createServer } from 'node:http';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createHash, generateKeyPairSync, sign, verify, randomBytes } from 'node:crypto';
import { deflateRawSync, inflateRawSync } from 'node:zlib';
const root=resolve(__dirname,'../../..');
const MAX_ZIP=25*1024*1024,MAX_TOTAL=64*1024*1024,MAX_ENTRY=20*1024*1024;
function crc32(bytes){let crc=0xffffffff;for(const byte of bytes){crc^=byte;for(let i=0;i<8;i++)crc=(crc>>>1)^((crc&1)?0xedb88320:0)}return (crc^0xffffffff)>>>0;}
function zip(entries){const local=[],central=[];let offset=0;
 for(const entry of entries){const name=Buffer.from(entry.name);const data=Buffer.from(entry.data||'');const method=entry.method||0;const packed=method===8?deflateRawSync(data):data;const crc=crc32(data);const h=Buffer.alloc(30);h.writeUInt32LE(0x04034b50);h.writeUInt16LE(20,4);h.writeUInt16LE(entry.flags||0,6);h.writeUInt16LE(method,8);h.writeUInt32LE(crc,14);h.writeUInt32LE(packed.length,18);h.writeUInt32LE(data.length,22);h.writeUInt16LE(name.length,26);local.push(h,name,packed);
 const c=Buffer.alloc(46);c.writeUInt32LE(0x02014b50);c.writeUInt16LE(3*256+20,4);c.writeUInt16LE(20,6);c.writeUInt16LE(entry.flags||0,8);c.writeUInt16LE(method,10);c.writeUInt32LE(crc,16);c.writeUInt32LE(packed.length,20);c.writeUInt32LE(data.length,24);c.writeUInt16LE(name.length,28);c.writeUInt32LE(((entry.mode||0o100644)<<16)>>>0,38);c.writeUInt32LE(offset,42);central.push(c,name);offset+=h.length+name.length+packed.length;
 }const directory=Buffer.concat(central);const end=Buffer.alloc(22);end.writeUInt32LE(0x06054b50);end.writeUInt16LE(entries.length,8);end.writeUInt16LE(entries.length,10);end.writeUInt32LE(directory.length,12);end.writeUInt32LE(offset,16);return Buffer.concat([...local,directory,end]);
}
const {publicKey,privateKey}=generateKeyPairSync('ed25519');
const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
function descriptor(bytes){const d={schema_version:1,tool_id:'demo-calculator',version:'1.0.0',abi_version:'1.0',sha256:hash(bytes),signing_key_id:'ephemeral-test-key'};return {...d,signature:sign(null,Buffer.from(`kortex-artifact-v1\n${d.tool_id}\n${d.version}\n${d.abi_version}\n${d.sha256}\n`),privateKey).toString('base64')};}
function validateArchive(bytes,proof){
 if(bytes.length>MAX_ZIP)throw Error('ARCHIVE_SIZE');
 if(!proof?.signature)throw Error('MISSING_SIGNATURE');
 if(hash(bytes)!==proof.sha256)throw Error('CHECKSUM');
 const text=`kortex-artifact-v1\n${proof.tool_id}\n${proof.version}\n${proof.abi_version}\n${proof.sha256}\n`;
 if(proof.signing_key_id!=='ephemeral-test-key'||!verify(null,Buffer.from(text),publicKey,Buffer.from(proof.signature,'base64')))throw Error('SIGNATURE');
 const end=bytes.length-22;if(end<0||bytes.readUInt32LE(end)!==0x06054b50)throw Error('ZIP_END');
 if(bytes.readUInt16LE(end+4)||bytes.readUInt16LE(end+6)||bytes.readUInt16LE(end+8)!==bytes.readUInt16LE(end+10))throw Error('MULTIDISK');
 const count=bytes.readUInt16LE(end+10),size=bytes.readUInt32LE(end+12),start=bytes.readUInt32LE(end+16);if(count>128||start+size!==end)throw Error('DIRECTORY');
 let cursor=start,total=0;const names=new Set(),ranges=[];const output={};
 for(let i=0;i<count;i++){
  if(cursor+46>end||bytes.readUInt32LE(cursor)!==0x02014b50)throw Error('CENTRAL');
  const flags=bytes.readUInt16LE(cursor+8),method=bytes.readUInt16LE(cursor+10),crc=bytes.readUInt32LE(cursor+16),compressed=bytes.readUInt32LE(cursor+20),length=bytes.readUInt32LE(cursor+24),n=bytes.readUInt16LE(cursor+28),extra=bytes.readUInt16LE(cursor+30),comment=bytes.readUInt16LE(cursor+32),mode=bytes.readUInt32LE(cursor+38)>>>16,offset=bytes.readUInt32LE(cursor+42);
  if(cursor+46+n+extra+comment>end)throw Error('CENTRAL_SIZE');
  const name=new TextDecoder('utf-8',{fatal:true}).decode(bytes.subarray(cursor+46,cursor+46+n));cursor+=46+n+extra+comment;
  if(flags!==0&&flags!==0x800)throw Error('ENCRYPTED_OR_UNSUPPORTED_FLAGS');if(![0,8].includes(method))throw Error('COMPRESSION');
  if((mode&0o170000)===0o120000)throw Error('SYMLINK');
  if(!/^[A-Za-z0-9_./-]+$/.test(name)||name.startsWith('/')||name.split('/').some(x=>!x||x==='.'||x==='..'||x.startsWith('.')))throw Error('PATH');
  if(!['manifest.json','contract.json','module.wasm','assets/icon.svg'].includes(name))throw Error('UNDECLARED_FILE');
  if(names.has(name))throw Error('DUPLICATE');names.add(name);
  total+=length;if(length>MAX_ENTRY||total>MAX_TOTAL||length>Math.max(1,compressed)*100)throw Error('DECOMPRESSION_BUDGET');
  if(offset+30>start||bytes.readUInt32LE(offset)!==0x04034b50)throw Error('LOCAL');
  const ln=bytes.readUInt16LE(offset+26),le=bytes.readUInt16LE(offset+28),begin=offset+30+ln+le,finish=begin+compressed;
  if(finish>start||bytes.readUInt16LE(offset+6)!==flags||bytes.readUInt16LE(offset+8)!==method||bytes.readUInt32LE(offset+14)!==crc||bytes.readUInt32LE(offset+18)!==compressed||bytes.readUInt32LE(offset+22)!==length||!bytes.subarray(offset+30,offset+30+ln).equals(Buffer.from(name)))throw Error('LOCAL_CENTRAL_MISMATCH');
  if(ranges.some(([a,b])=>offset<b&&finish>a))throw Error('OVERLAP');ranges.push([offset,finish]);
  const packed=bytes.subarray(begin,finish);const data=method===0?Buffer.from(packed):inflateRawSync(packed,{maxOutputLength:Math.max(1,length)});
  if(data.length!==length||crc32(data)!==crc)throw Error('CRC_OR_LENGTH');output[name]=data;
 }
 if(cursor!==end||!['manifest.json','contract.json','module.wasm'].every(x=>names.has(x)))throw Error('MISSING_ENTRY');return {files:Object.keys(output),total};
}
const referenceEntries=[{name:'manifest.json',data:readFileSync(resolve(root,'shared/fixtures/manifest.valid.json'))},{name:'contract.json',data:readFileSync(resolve(root,'shared/fixtures/contract.valid.json'))},{name:'module.wasm',data:readFileSync(resolve(root,'tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm'))}];
const reference=zip(referenceEntries);
test('ZIP: signature, checksum and hostile metadata are rejected before extraction',()=>{
 const start=process.memoryUsage();expect(validateArchive(reference,descriptor(reference)).files.length).toBe(3);
 expect(()=>validateArchive(reference,null)).toThrow('MISSING_SIGNATURE');expect(()=>validateArchive(reference,{...descriptor(reference),signature:Buffer.alloc(64).toString('base64')})).toThrow('SIGNATURE');
 const altered=Buffer.from(reference);altered[40]^=1;expect(()=>validateArchive(altered,descriptor(reference))).toThrow('CHECKSUM');
 for(const entry of [{name:'../escape',data:'x'},{name:'assets/../escape',data:'x'},{name:'.hidden',data:'x'},{name:'link',data:'target',mode:0o120777},{name:'module.wasm',data:'x',flags:1},{name:'assets/icon.svg',data:Buffer.alloc(512*1024,65),method:8}]){const archive=zip([...referenceEntries,entry]);expect(()=>validateArchive(archive,descriptor(archive))).toThrow();}
 const oversized=Buffer.alloc(MAX_ZIP+1);expect(()=>validateArchive(oversized,null)).toThrow('ARCHIVE_SIZE');
 const duplicate=zip([...referenceEntries,referenceEntries[0]]);expect(()=>validateArchive(duplicate,descriptor(duplicate))).toThrow('DUPLICATE');
 const mismatched=Buffer.from(reference);mismatched.writeUInt32LE(42,18);expect(()=>validateArchive(mismatched,descriptor(mismatched))).toThrow('LOCAL_CENTRAL_MISMATCH');
 const badCrc=Buffer.from(reference);badCrc[30+Buffer.byteLength('manifest.json')+1]^=1;expect(()=>validateArchive(badCrc,descriptor(badCrc))).toThrow('CRC_OR_LENGTH');
 const deflated=zip(referenceEntries.map(e=>({...e,method:8})));expect(validateArchive(deflated,descriptor(deflated)).files.length).toBe(3);
 console.log('archive memory sample',JSON.stringify({before:start,after:process.memoryUsage(),reference_bytes:reference.length,deflated_bytes:deflated.length}));
});
let servers=[],originParent,originRuntime;
function server(role){return createServer((req,res)=>{
 const url=new URL(req.url,'http://local');const cspNonce=randomBytes(18).toString('base64');
 res.setHeader('Content-Security-Policy',`default-src 'none'; script-src 'self' 'nonce-${cspNonce}' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; frame-src ${originRuntime||"'none'"}; frame-ancestors ${role==='runtime'?originParent:"'none'"}`);
 if(url.pathname==='/reference.tool'){res.setHeader('Content-Type','application/zip');res.end(reference);return}
 if(url.pathname==='/module.wasm'){res.setHeader('Content-Type','application/wasm');res.end(referenceEntries[2].data);return}
 if(url.pathname==='/worker.js'){res.setHeader('Content-Type','text/javascript');res.end(`fetch('/module.wasm').then(r=>r.arrayBuffer()).then(b=>WebAssembly.compile(b)).then(m=>{if(WebAssembly.Module.imports(m).some(i=>i.module!=='kortex_v1'))throw Error('Unexpected import');return WebAssembly.instantiate(m,{kortex_v1:{host_enqueue:()=>-1}})}).then(i=>postMessage({exports:Object.keys(i.exports),dom:typeof document}));`);return}
 if(url.pathname==='/iframe.html'||url.pathname==='/index.html'){res.setHeader('Content-Type','text/html');res.end(readFileSync(resolve(__dirname,url.pathname.slice(1)),'utf8').replace('<script>',`<script nonce="${cspNonce}">`));return}
 res.statusCode=404;res.end();
})}
test.beforeAll(async()=>{for(const role of ['parent','runtime']){const s=server(role);await new Promise(resolve=>s.listen(0,'127.0.0.1',resolve));servers.push(s);const origin=`http://127.0.0.1:${s.address().port}`;if(role==='parent')originParent=origin;else originRuntime=origin;}});
test.afterAll(async()=>{for(const s of servers)await new Promise(resolve=>s.close(resolve));});
for(const [name,type] of Object.entries({chromium,webkit}))test(`${name}: actual cross-origin sandbox and Worker`,async()=>{
 const browser=await type.launch({headless:true});try{const page=await browser.newPage();await page.goto(originParent+'/index.html?runtime='+encodeURIComponent(originRuntime));await expect.poll(()=>page.evaluate(()=>window.received.length)).toBe(1);
 const iframe=page.frames().find(f=>f.url().startsWith(originRuntime));expect(iframe).toBeTruthy();
 expect(await page.locator('iframe').getAttribute('sandbox')).toBe('allow-scripts allow-same-origin');
 expect(await iframe.evaluate(()=>{try{parent.document.body;return 'accessible'}catch(e){return e.name}})).toBe('SecurityError');
 await page.evaluate(()=>window.sendProbe());await expect.poll(()=>iframe.evaluate(()=>window.accepted)).toBe(1);
 // Wrong nonce with correct source/origin.
 await page.evaluate(()=>{document.querySelector('iframe').contentWindow.postMessage({kind:'probe',nonce:'forged'},window.testConfig.runtime)});await expect.poll(()=>iframe.evaluate(()=>window.rejected)).toBe(1);
 // Same-origin runtime sibling: origin and nonce match but event.source is not parent.
 const nonce=await page.evaluate(()=>window.testConfig.nonce);const attacker=await browser.newPage();await attacker.goto(originRuntime+'/iframe.html?parent='+encodeURIComponent(originParent)+'&nonce=attacker');
 await page.evaluate(({nonce})=>{const sibling=document.createElement('iframe');sibling.src=window.testConfig.runtime+'/iframe.html?parent='+encodeURIComponent(location.origin)+'&nonce='+nonce;document.body.append(sibling)},{nonce});await expect.poll(()=>page.evaluate(()=>window.rejected)).toBeGreaterThan(0);
 await iframe.evaluate(({parentOrigin,nonce})=>{window.postMessage({kind:'probe',nonce},location.origin)},{parentOrigin:originParent,nonce});await expect.poll(()=>iframe.evaluate(()=>window.rejected)).toBeGreaterThan(1);
 // CSP forbids outbound requests and iframe top navigation is not granted.
 expect(await iframe.evaluate(async()=>{try{await fetch('https://example.invalid');return 'allowed'}catch{return 'blocked'}})).toBe('blocked');
 expect(await iframe.evaluate(()=>{try{top.location.href=location.origin+'/iframe.html';return 'allowed'}catch{return 'blocked'}})).toBe('blocked');
 await iframe.evaluate(()=>{const script=document.createElement('script');script.textContent='window.injectedScriptExecuted=true';document.body.append(script)});expect(await iframe.evaluate(()=>window.injectedScriptExecuted)).toBeUndefined();
 const download=await iframe.evaluate(()=>window.download());expect(download.sha256).toBe(hash(reference));expect(download.bytes).toBe(reference.length);
 const worker=await iframe.evaluate(()=>window.workerProbe());expect(worker.dom).toBe('undefined');expect(worker.exports).toContain('kortex_dispatch');await attacker.close();
 }finally{await browser.close();}
});
