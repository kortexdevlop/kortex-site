import {LIMIT,MEMORY_LIMIT} from './types';
export function range(memory:WebAssembly.Memory,ptr:number,len:number):void {
 if(!Number.isInteger(ptr)||!Number.isInteger(len)||ptr<0||len<=0||len>LIMIT||ptr+len>0xffffffff||ptr+len>memory.buffer.byteLength||memory.buffer.byteLength>MEMORY_LIMIT)throw Error('INVALID_BUFFER');
}
export function unpack(packed:bigint):[number,number] {const bits=BigInt.asUintN(64,packed);return [Number(bits>>32n),Number(bits&0xffffffffn)];}
/// Reject missing/excess/shared/memory64 maxima before instantiation, including start functions.
export function inspectMemory(bytes:Uint8Array):void {
 let at=8,count=0;
 if(bytes.length<8||new DataView(bytes.buffer,bytes.byteOffset,8).getUint32(0,true)!==0x6d736100)throw Error('INVALID_MODULE');
 const read=()=>{let n=0,shift=0;for(let i=0;i<5;i++){if(at>=bytes.length)throw Error('INVALID_MODULE');const b=bytes[at++];if(i===4&&(b&0xf0))throw Error('INVALID_MODULE');n+=(b&127)*2**shift;if(!(b&128))return n;shift+=7;}throw Error('INVALID_MODULE')};
 while(at<bytes.length){const id=bytes[at++],size=read(),end=at+size;if(end>bytes.length)throw Error('INVALID_MODULE');if(id===5){const n=read();if(n!==1)throw Error('INVALID_MEMORY');count=n;const flags=read();if(flags!==1)throw Error('INVALID_MEMORY');const min=read(),max=read();if(min>max||max>1024)throw Error('INVALID_MEMORY');if(at!==end)throw Error('INVALID_MODULE');}at=end;}
 if(count!==1)throw Error('INVALID_MEMORY');
}
