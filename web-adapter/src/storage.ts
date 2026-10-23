const QUOTA=5*1024*1024;
type Entry={version:number;key:string;value:string};type RecordValue={revision:number;entries:Entry[]};
function failure(e:unknown):Error {if(e instanceof Error&&['QUOTA','PERMISSION','CANCELLED','CORRUPT','UNAVAILABLE'].includes(e.message))return e;const name=(e as DOMException)?.name;return Error(name==='QuotaExceededError'?'QUOTA':name==='SecurityError'?'PERMISSION':name==='AbortError'?'CANCELLED':'UNAVAILABLE');}
export function storageKey(key:string){if(typeof key!=='string'||key.length<1||key.length>256||!/^[A-Za-z0-9_./-]+$/.test(key)||key.split('/').some(p=>!p||p==='.'||p==='..'))throw Error('CORRUPT');}
function validate(record:RecordValue){if(!Number.isSafeInteger(record.revision)||record.revision<0||!Array.isArray(record.entries))throw Error('CORRUPT');let size=0;const keys=new Set<string>();for(const e of record.entries){storageKey(e.key);if(!Number.isSafeInteger(e.version)||e.version<1||typeof e.value!=='string')throw Error('CORRUPT');const id=JSON.stringify([e.version,e.key]);if(keys.has(id))throw Error('CORRUPT');keys.add(id);try{JSON.parse(e.value)}catch{throw Error('CORRUPT')};size+=new TextEncoder().encode(e.key+e.value).length;if(size>QUOTA)throw Error('QUOTA');}}
/** Real IndexedDB backend, restricted to a dedicated tool-host origin. Never stores credentials. */
export class IndexedToolStorage {
 private database:Promise<IDBDatabase>|undefined;private owner:string;
 constructor(profile:string,tool_id:string,private version:number,private dedicatedOrigin:string,private parentOrigin:string){if(!profile||!tool_id||!Number.isSafeInteger(version)||version<1)throw Error('PERMISSION');this.owner=JSON.stringify([profile,tool_id]);}
 async requestPersistence(){
  if(!navigator.userActivation?.isActive)throw Error('PERMISSION');
  try{if(await navigator.storage?.persisted?.())return;if(!navigator.storage?.persist||!await navigator.storage.persist())throw Error('UNAVAILABLE')}catch{throw Error('UNAVAILABLE')}
 }
 private async db(){
  if(location.origin!==this.dedicatedOrigin||this.parentOrigin===this.dedicatedOrigin)throw Error('PERMISSION');
  // There is no reliable private-mode detector. Refuse durable state unless the browser
  // confirms persistent storage, including private/unsupported/evictable contexts.
  let persisted=false;try{persisted=!!navigator.storage?.persisted&&await navigator.storage.persisted()}catch{throw Error('UNAVAILABLE')}if(!persisted)throw Error('UNAVAILABLE');
  return this.database??=new Promise<IDBDatabase>((resolve,reject)=>{const req=indexedDB.open('kortex-tool-state-v1',1);req.onupgradeneeded=()=>{req.result.createObjectStore('tools')};req.onerror=()=>{this.database=undefined;reject(failure(req.error))};req.onblocked=()=>{this.database=undefined;reject(Error('UNAVAILABLE'))};req.onsuccess=()=>{const db=req.result;db.onversionchange=()=>{db.close();this.database=undefined};resolve(db)}});
 }
 private async transaction<T>(write:boolean,edit:(record:RecordValue)=>T,signal?:AbortSignal):Promise<T>{
  if(signal?.aborted)throw Error('CANCELLED');const db=await this.db();if(signal?.aborted)throw Error('CANCELLED');
  return new Promise((resolve,reject)=>{let tx:IDBTransaction;try{tx=db.transaction('tools',write?'readwrite':'readonly')}catch(e){reject(failure(e));return}const store=tx.objectStore('tools');let result:T;let error:Error|undefined;
   const abort=()=>tx.abort();signal?.addEventListener('abort',abort,{once:true});
   tx.oncomplete=()=>{signal?.removeEventListener('abort',abort);resolve(result)};tx.onabort=()=>{signal?.removeEventListener('abort',abort);reject(error??failure(tx.error??new DOMException('abort','AbortError')))};tx.onerror=()=>{};
   const request=store.get(this.owner);request.onerror=()=>{error=failure(request.error)};
   request.onsuccess=()=>{try{const record:RecordValue=request.result??{revision:0,entries:[]};validate(record);result=edit(record);if(write){validate(record);record.revision++;if(!Number.isSafeInteger(record.revision))throw Error('UNAVAILABLE');store.put(record,this.owner)}}catch(e){error=failure(e);tx.abort();}};
  });
 }
 get(key:string,signal?:AbortSignal){storageKey(key);return this.transaction(false,r=>{const v=r.entries.find(e=>e.version===this.version&&e.key===key);return v?JSON.parse(v.value):null},signal);}
 set(key:string,value:unknown,signal?:AbortSignal){storageKey(key);const encoded=JSON.stringify(value);if(encoded===undefined)throw Error('CORRUPT');if(new TextEncoder().encode(encoded).length>QUOTA)throw Error('QUOTA');return this.transaction(true,r=>{r.entries=r.entries.filter(e=>e.version!==this.version||e.key!==key);r.entries.push({version:this.version,key,value:encoded});return {}},signal);}
 delete(key:string,signal?:AbortSignal){storageKey(key);return this.transaction(true,r=>{r.entries=r.entries.filter(e=>e.version!==this.version||e.key!==key);return {}},signal);}
 clearNamespace(signal?:AbortSignal){return this.transaction(true,r=>{r.entries=r.entries.filter(e=>e.version!==this.version);return {}},signal);}
 close(){void this.database?.then(db=>db.close());this.database=undefined;}
}
