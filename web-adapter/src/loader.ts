import {Revocations} from './revocation';
import {TrustStore} from './trust';
import {VerifiedTool,type Descriptor} from './bridge';
export interface ArtifactDescriptor extends Descriptor {url:string}
export class ToolLoader {
 constructor(private trust:TrustStore,private apiOrigin:string,private changed:(state:string)=>void=()=>{},private revocations?:Revocations){const u=new URL(apiOrigin);if(u.origin!==apiOrigin||!(u.protocol==='https:'||u.protocol==='http:'&&['localhost','127.0.0.1'].includes(u.hostname)))throw Error('INVALID_ORIGIN')}
 private cacheKey(proof:Descriptor){return `${location.origin}/__kortex_artifacts/current`}
 async loadTool(descriptor:ArtifactDescriptor,signal?:AbortSignal):Promise<VerifiedTool>{
  const {url:downloadUrl,...proof}=structuredClone(descriptor);const url=new URL(downloadUrl);if(url.origin!==this.apiOrigin||url.username||url.password||url.hash)throw Error('INVALID_ORIGIN');
  await this.revocations?.authorize(this.apiOrigin,proof,signal);
  const timeout=AbortSignal.timeout(30000),cancel=signal?AbortSignal.any([signal,timeout]):timeout;const cache=await caches.open('kortex-artifacts-v1');const key=this.cacheKey(proof);this.changed('verifying');
  let cached=await cache.match(key);if(cached?.headers.get('x-kortex-sha256')&&cached.headers.get('x-kortex-sha256')!==proof.sha256){await cache.delete(key);cached=undefined}if(cached){try{const bytes=await read(cached,cancel);const tool=await VerifiedTool.verifyArchive(bytes,proof,this.trust);if(cancel.aborted)throw Error('CANCELLED');this.revocations?.check(proof);return tool}catch(e){await cache.delete(key);throw e}}
  this.changed('downloading');const response=await fetch(url,{redirect:'error',credentials:'omit',referrerPolicy:'no-referrer',signal:cancel,cache:'no-store'});if(!response.ok||!['application/vnd.kortex.tool+zip','application/zip','application/octet-stream'].includes((response.headers.get('content-type')??'').split(';')[0].trim()))throw Error('INVALID_DOWNLOAD');
  const bytes=await read(response,cancel);this.changed('verifying');const tool=await VerifiedTool.verifyArchive(bytes,proof,this.trust);if(cancel.aborted)throw Error('CANCELLED');
  // The cache contains only verified archives, is bounded, and every hit is verified again.
  const keys=await cache.keys();for(const old of keys)await cache.delete(old);await cache.put(key,new Response(bytes,{headers:{'x-kortex-sha256':proof.sha256}}));this.revocations?.check(proof);return tool;
 }
 async uninstall(proof:Descriptor){const cache=await caches.open('kortex-artifacts-v1');await cache.delete(this.cacheKey(proof));}
}
async function read(response:Response,signal:AbortSignal){if(Number(response.headers.get('content-length'))>25*1024*1024)throw Error('SIZE_LIMIT');const reader=response.body?.getReader();if(!reader)throw Error('INVALID_DOWNLOAD');const chunks:Uint8Array[]=[];let length=0;const abort=()=>{void reader.cancel()};signal.addEventListener('abort',abort,{once:true});try{for(;;){if(signal.aborted)throw Error('CANCELLED');const item=await reader.read();if(item.done)break;length+=item.value.length;if(length>25*1024*1024){await reader.cancel();throw Error('SIZE_LIMIT')}chunks.push(item.value)}}finally{signal.removeEventListener('abort',abort);reader.releaseLock()}if(signal.aborted)throw Error('CANCELLED');const result=new Uint8Array(length);let p=0;for(const chunk of chunks){result.set(chunk,p);p+=chunk.length}return result}
