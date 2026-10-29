import {z} from 'zod';
type Environment='local'|'staging'|'production';
export type IdentityConfig={issuer:string;webClientId:string;desktopClientId:string;webRedirect:string;desktopRedirect:string;endpointOrigins:string[]};
function secure(value:string,local:boolean):URL {
 const u=new URL(value);
 if(u.username||u.password||u.hash||(u.protocol!=='https:'&&!(local&&u.protocol==='http:'&&['localhost','127.0.0.1','[::1]'].includes(u.hostname))))throw Error('IDENTITY_URL');
 return u;
}
export function identityConfig(raw:Record<string,unknown>,environment:Environment):IdentityConfig|null {
 if(raw.OIDC_ENABLED!=='true')return null;
 const local=environment==='local';
 const required=(name:string)=>{const v=raw[name];if(typeof v!=='string'||!v.trim()||v.length>2048)throw Error('IDENTITY_CONFIG');return v};
 const issuer=required('OIDC_ISSUER'),issuerUrl=secure(issuer,local);if(issuerUrl.search)throw Error('IDENTITY_ISSUER');
 const webRedirect=required('OIDC_WEB_REDIRECT_URI'),desktopRedirect=required('OIDC_DESKTOP_REDIRECT_URI');
 secure(webRedirect,local);const desktop=secure(desktopRedirect,true);
 if(!['localhost','127.0.0.1','[::1]'].includes(desktop.hostname)||desktop.protocol!=='http:')throw Error('IDENTITY_DESKTOP_REDIRECT');
 const origins=required('OIDC_ENDPOINT_ORIGINS').split(',');for(const origin of origins)if(secure(origin,local).origin!==origin)throw Error('IDENTITY_ORIGIN');
 if(!origins.includes(issuerUrl.origin))throw Error('IDENTITY_ORIGIN');
 return {issuer,webClientId:required('OIDC_WEB_CLIENT_ID'),desktopClientId:required('OIDC_DESKTOP_CLIENT_ID'),webRedirect,desktopRedirect,endpointOrigins:origins};
}
const discoverySchema=z.object({issuer:z.string(),authorization_endpoint:z.string(),token_endpoint:z.string(),jwks_uri:z.string(),response_types_supported:z.array(z.string()),subject_types_supported:z.array(z.string()),id_token_signing_alg_values_supported:z.array(z.string()),code_challenge_methods_supported:z.array(z.string())});
async function boundedJson(response:Response):Promise<unknown>{
 if(!response.ok||!response.body)throw Error('IDENTITY_DISCOVERY');
 const reader=response.body.getReader();let size=0;const chunks:Uint8Array[]=[];
 try{for(;;){const part=await reader.read();if(part.done)break;size+=part.value.length;if(size>65536)throw Error('IDENTITY_SIZE');chunks.push(part.value)}}finally{await reader.cancel()}
 const bytes=new Uint8Array(size);let at=0;for(const chunk of chunks){bytes.set(chunk,at);at+=chunk.length}return JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(bytes));
}
export async function discoverIdentity(config:IdentityConfig,environment:Environment,transport:typeof fetch=fetch){
 const local=environment==='local';const endpoint=(value:string)=>{const u=secure(value,local);if(!config.endpointOrigins.includes(u.origin))throw Error('IDENTITY_ENDPOINT');return u.href};
 const load=async(url:string)=>boundedJson(await transport(endpoint(url),{redirect:'manual',signal:AbortSignal.timeout(10000),headers:{Accept:'application/json'}}));
 const metadata=discoverySchema.parse(await load(config.issuer.replace(/\/$/,'')+'/.well-known/openid-configuration'));
 if(metadata.issuer!==config.issuer||!metadata.response_types_supported.includes('code')||!metadata.code_challenge_methods_supported.includes('S256')||!metadata.id_token_signing_alg_values_supported.some(a=>['RS256','ES256','EdDSA'].includes(a)))throw Error('IDENTITY_METADATA');
 endpoint(metadata.authorization_endpoint);endpoint(metadata.token_endpoint);endpoint(metadata.jwks_uri);
 const jwks=z.object({keys:z.array(z.record(z.string(),z.unknown())).min(1).max(20)}).parse(await load(metadata.jwks_uri));
 for(const key of jwks.keys)if(!['RSA','EC','OKP'].includes(String(key.kty))||'d'in key||'k'in key||key.alg==='none'||key.use!==undefined&&key.use!=='sig')throw Error('IDENTITY_JWKS');
 return {metadata,jwks};
}
