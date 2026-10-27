/** The configuration is selected by the host deployment, never taken from a message. */
export interface EmbedConfig {parentOrigin:string;clientVersion:string}
export class EmbedSession {
 private nonce:string|undefined;private session:string|undefined;private active=false;
 constructor(private config:EmbedConfig,private open:(tool_id:string,version:string)=>Promise<unknown>,private changed:(state:string)=>void=()=>{}){
  const parent=new URL(config.parentOrigin);if(parent.origin!==config.parentOrigin||parent.origin===location.origin||!['https:','http:'].includes(parent.protocol))throw Error('INVALID_PARENT_ORIGIN');
 }
 handle(event:MessageEvent){
  if(event.source!==window.parent||event.origin!==this.config.parentOrigin||window.parent===window)return;
  const data=event.data;if(!data||typeof data!=='object'||Array.isArray(data))return;
  if(data.kind==='HELLO'){
   if(this.nonce||typeof data.nonce!=='string'||!/^[a-f0-9]{64}$/.test(data.nonce)||data.client_version!==this.config.clientVersion)return;
   this.nonce=data.nonce;this.session=crypto.randomUUID();window.parent.postMessage({kind:'READY',nonce:this.nonce,session_id:this.session},this.config.parentOrigin);return;
  }
  if(data.kind!=='OPEN'||!this.session||data.session_id!==this.session||data.nonce!==this.nonce||this.active||typeof data.tool_id!=='string'||!/^[a-z][a-z0-9-]{0,63}$/.test(data.tool_id)||typeof data.version!=='string'||data.version.length>100)return;
  this.active=true;this.changed('loading');void this.open(data.tool_id,data.version).then(result=>{window.parent.postMessage({kind:'RESULT',session_id:this.session,nonce:this.nonce,result},this.config.parentOrigin);this.changed('ready')},()=>{window.parent.postMessage({kind:'ERROR',session_id:this.session,nonce:this.nonce,code:'OPEN_FAILED'},this.config.parentOrigin);this.changed('error')}).finally(()=>{this.active=false});
 }
 listen(){const listener=(e:MessageEvent)=>this.handle(e);window.addEventListener('message',listener);return ()=>window.removeEventListener('message',listener);}
}
/** HTTP response headers must be installed by the dedicated origin's server. */
export function frameHeaders(parentOrigin:string){const p=new URL(parentOrigin);if(p.origin!==parentOrigin)throw Error('INVALID_PARENT_ORIGIN');return {
 'Content-Security-Policy':`default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'; img-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors ${parentOrigin}`,
 'Referrer-Policy':'no-referrer','X-Content-Type-Options':'nosniff','Permissions-Policy':'camera=(), microphone=(), geolocation=()'
};}
