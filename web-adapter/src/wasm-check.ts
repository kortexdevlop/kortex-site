// Static ABI inspection. No allocation of guest memory and no instantiation.
export function inspectWasm(bytes:Uint8Array):void {
 const bad=():never=>{throw Error('INVALID_WASM_ABI')};if(bytes.length>20*1024*1024||[0,97,115,109,1,0,0,0].some((v,i)=>bytes[i]!==v))bad();
 let at=8,limit=bytes.length;const byte=()=>{if(at>=limit)bad();return bytes[at++]};
 const uint=()=>{let n=0;for(let i=0;i<5;i++){const b=byte();if(i===4&&(b&240))bad();n+=(b&127)*2**(7*i);if(!(b&128))return n}return bad()};
 const text=()=>{const n=uint();if(n>256||at+n>limit)bad();const s=new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(bytes.subarray(at,at+n));at+=n;return s};
 const values=()=>{const n=uint();if(n>100000||at+n>limit)bad();const a=[];for(let i=0;i<n;i++){const v=byte();if(![127,126,125,124,123,112,111].includes(v))bad();a.push(v)}return a};
 const types:{p:number[];r:number[]}[]=[],funcs:number[]=[],exports=new Map<string,{kind:number,index:number}>();let memories=0,imports=0;
 while(at<bytes.length){limit=bytes.length;const id=byte(),size=uint(),end=at+size;if(end>bytes.length)bad();limit=end;
  if(id===1){const n=uint();if(n>100000)bad();for(let i=0;i<n;i++){if(byte()!==96)bad();types.push({p:values(),r:values()})}}
  else if(id===2){const n=uint();if(n>1)bad();for(let i=0;i<n;i++){if(text()!=='kortex_v1'||text()!=='host_enqueue'||byte()!==0)bad();funcs.push(uint());imports++}}
  else if(id===3){const n=uint();if(n>100000)bad();for(let i=0;i<n;i++)funcs.push(uint())}
  else if(id===4){const n=uint();if(n>1)bad();for(let i=0;i<n;i++){if(byte()!==112||uint()!==1)bad();const min=uint(),max=uint();if(min>max||max>65536)bad()}}
  else if(id===5){memories=uint();if(memories!==1||uint()!==1)bad();const min=uint(),max=uint();if(min>max||max>1024)bad()}
  else if(id===7){const n=uint();if(n>100000)bad();for(let i=0;i<n;i++){const name=text(),kind=byte(),index=uint();if(exports.has(name))bad();exports.set(name,{kind,index})}}
  else if(id===8)bad();
  else {at=end;}
  if(at!==end)bad();at=end;
 }
 const eq=(a:number[]|undefined,b:number[])=>a?.join(',')===b.join(',');
 const signature=(index:number,p:number[],r:number[])=>{const t=types[funcs[index]];if(!t||!eq(t.p,p)||!eq(t.r,r))bad()};
 if(memories!==1||exports.get('memory')?.kind!==2||exports.get('memory')?.index!==0)bad();if(imports)signature(0,[127,127],[127]);
 for(const[name,p,r]of [['kortex_alloc',[127],[127]],['kortex_dealloc',[127,127],[]],['kortex_dispatch',[127,127],[126]]] as const){const e=exports.get(name);if(!e||e.kind!==0)bad();signature(e!.index,[...p],[...r])}
 for(const[name,e]of exports)if(!['memory','kortex_alloc','kortex_dealloc','kortex_dispatch'].includes(name)&&!(['__heap_base','__data_end'].includes(name)&&e.kind===3))bad();
}
