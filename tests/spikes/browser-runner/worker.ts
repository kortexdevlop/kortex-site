// JS-compatible TypeScript served directly as a module; no hidden ABI adaptation.
let instance;let denied=0;let allocations=0;
const limit=1048576;
function range(ptr,len){const size=instance.exports.memory.buffer.byteLength;if(!Number.isInteger(ptr)||!Number.isInteger(len)||len<=0||len>limit||ptr+len>size||ptr+len>0xffffffff)throw Error('Invalid ABI range');}
async function load(){
 const bytes=await(await fetch('/tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm')).arrayBuffer();
 const module=await WebAssembly.compile(bytes);
 const imports=WebAssembly.Module.imports(module);if(imports.some(i=>i.module!=='kortex_v1'||i.name!=='host_enqueue'))throw Error('Unexpected import');
 instance=await WebAssembly.instantiate(module,{kortex_v1:{host_enqueue(ptr,len){range(ptr>>>0,len>>>0);denied++;return -1;}}});
 if(instance.exports.memory.buffer.byteLength>67108864)throw Error('Memory limit');
 return {imports,exports:WebAssembly.Module.exports(module),bytes:bytes.byteLength};
}
function dispatch(request,raw){
 const bytes=new TextEncoder().encode(raw===undefined?JSON.stringify(request):raw);if(bytes.length>limit)throw Error('Message exceeds 1 MiB');
 const ptr=instance.exports.kortex_alloc(bytes.length)>>>0;if(!ptr)throw Error('Allocation refused');allocations++;
 let responsePtr=0,responseLen=0;
 try{
  range(ptr,bytes.length);new Uint8Array(instance.exports.memory.buffer,ptr,bytes.length).set(bytes);
  const packed=BigInt.asUintN(64,instance.exports.kortex_dispatch(ptr,bytes.length));responsePtr=Number(packed>>32n);responseLen=Number(packed&0xffffffffn);range(responsePtr,responseLen);allocations++;
  const copy=new Uint8Array(instance.exports.memory.buffer,responsePtr,responseLen).slice();return JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(copy));
 }finally{if(responsePtr){instance.exports.kortex_dealloc(responsePtr,responseLen);allocations--;}instance.exports.kortex_dealloc(ptr,bytes.length);allocations--;}
}
self.onmessage=async({data})=>{try{let result;if(data.op==='load')result=await load();else if(data.op==='repeat'){const start=performance.now();for(let i=0;i<data.count;i++){const r=dispatch(data.request);if(!r.ok||r.data.result!==1200||r.request_id!==data.request.request_id)throw Error('Dispatch mismatch');}result={count:data.count,ms:performance.now()-start,allocations,denied,memory:instance.exports.memory.buffer.byteLength};}else if(data.op==='oversized-guest'){const packed=BigInt.asUintN(64,instance.exports.kortex_dispatch(0,limit+1));const p=Number(packed>>32n),n=Number(packed&0xffffffffn);range(p,n);try{result=JSON.parse(new TextDecoder().decode(new Uint8Array(instance.exports.memory.buffer,p,n)))}finally{instance.exports.kortex_dealloc(p,n)}}else result=dispatch(data.request,data.raw);self.postMessage({id:data.id,result});}catch(e){self.postMessage({id:data.id,error:e.message});}};
