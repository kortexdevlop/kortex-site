const fail=():never=>{throw Error('INVALID_ARCHIVE')};
export async function extractArchive(bytes:Uint8Array):Promise<Map<string,Uint8Array>>{
 if(bytes.length>25*1024*1024||bytes.length<22)fail();const v=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);const u16=(p:number)=>v.getUint16(p,true),u32=(p:number)=>v.getUint32(p,true);
 let end=-1;for(let p=bytes.length-22;p>=Math.max(0,bytes.length-65557);p--)if(u32(p)===0x06054b50&&p+22+u16(p+20)===bytes.length){end=p;break}if(end<0)fail();
 if(u16(end+4)||u16(end+6)||u16(end+8)!==u16(end+10))fail();const count=u16(end+10),offset=u32(end+16),size=u32(end+12);if(count<1||count>256||offset+size!==end)fail();
 const entries:{name:string;start:number;compressed:number;length:number;method:number;crc:number}[]=[];let at=offset,total=0;const names=new Set<string>();const ranges:[number,number][]=[];
 for(let i=0;i<count;i++){
  if(at+46>end||u32(at)!==0x02014b50)fail();const flags=u16(at+8),method=u16(at+10),crc=u32(at+16),compressed=u32(at+20),length=u32(at+24),nl=u16(at+28),extra=u16(at+30),comment=u16(at+32),local=u32(at+42);
  if(at+46+nl+extra+comment>end||flags&~0x800||![0,8].includes(method)||u16(at+34)||((u32(at+38)>>>16)&0xf000)===0xa000)fail();
  const name=new TextDecoder('utf-8',{fatal:true}).decode(bytes.subarray(at+46,at+46+nl));
  if(!name||name.length>256||!/^[A-Za-z0-9_./-]+$/.test(name)||name.startsWith('/')||name.split('/').some(p=>!p||p==='.'||p==='..')||names.has(name.toLowerCase()))fail();names.add(name.toLowerCase());
  total+=length;if(total>50*1024*1024||length>20*1024*1024||length>Math.max(1,compressed)*100)throw Error('ARCHIVE_LIMIT');
  if(local+30>offset||u32(local)!==0x04034b50||u16(local+6)!==flags||u16(local+8)!==method||u32(local+14)!==crc||u32(local+18)!==compressed||u32(local+22)!==length||u16(local+26)!==nl)fail();
  const start=local+30+nl+u16(local+28);if(start+compressed>offset||new TextDecoder().decode(bytes.subarray(local+30,local+30+nl))!==name)fail();
  for(const [a,b] of ranges)if(local<b&&a<start+compressed)fail();ranges.push([local,start+compressed]);entries.push({name,start,compressed,length,method,crc});at+=46+nl+extra+comment;
 }
 if(at!==end)fail();const result=new Map<string,Uint8Array>();
 for(const entry of entries){let data=bytes.slice(entry.start,entry.start+entry.compressed);
  if(entry.method===8){const reader=new Blob([data]).stream().pipeThrough(new DecompressionStream('deflate-raw')).getReader();const parts:Uint8Array[]=[];let n=0;try{for(;;){const r=await reader.read();if(r.done)break;n+=r.value.length;if(n>entry.length){await reader.cancel();throw Error('ARCHIVE_LIMIT')}parts.push(r.value)}}finally{reader.releaseLock()}data=new Uint8Array(n);let p=0;for(const part of parts){data.set(part,p);p+=part.length}}
  if(data.length!==entry.length||crc32(data)!==entry.crc)fail();result.set(entry.name,data);
 }
 for(const name of ['manifest.json','contract.json','module.wasm'])if(!result.has(name))fail();return result;
}
export function crc32(bytes:Uint8Array):number{let crc=0xffffffff;for(const b of bytes){crc^=b;for(let i=0;i<8;i++)crc=(crc>>>1)^((crc&1)?0xedb88320:0)}return (crc^0xffffffff)>>>0}
