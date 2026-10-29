import {z} from 'zod';
import type {Env} from '../env';
import {base64url} from '../security/admin-auth';
import {readBounded} from '../lib/admin-error';
import {identityConfig,discoverIdentity,type IdentityConfig} from './identity-config';
import {encoder} from '../security/sessions';
export type Discovery=Awaited<ReturnType<typeof discoverIdentity>>;
export function configuration(env:Env):IdentityConfig|null{
 const config=identityConfig(env as unknown as Record<string,unknown>,env.KORTEX_ENV);if(!config)return null;
 if(config.webRedirect!==env.PUBLIC_WEB_ORIGIN+'/auth/callback'&&config.webRedirect!==env.PUBLIC_WEB_ORIGIN+'/api/v1/auth/callback')throw Error('IDENTITY_REDIRECT');
 if(!env.OIDC_WEB_CLIENT_SECRET||env.OIDC_WEB_CLIENT_SECRET.length>4096)throw Error('IDENTITY_CONFIG');return config;
}
const headerSchema=z.object({alg:z.literal('RS256'),kid:z.string().min(1).max(128),typ:z.literal('JWT').optional()}).strict();
const claimsSchema=z.object({iss:z.string(),sub:z.string().min(1).max(255),aud:z.union([z.string(),z.array(z.string()).min(1).max(8)]),azp:z.string().optional(),exp:z.number().int(),iat:z.number().int(),nbf:z.number().int().optional(),nonce:z.string().min(1).max(256).optional()});
export async function verifyIdToken(token:string,config:IdentityConfig,discovery:Discovery,audience:string,now=Math.floor(Date.now()/1000)){
 if(token.length>16384)throw Error('IDENTITY_TOKEN');const parts=token.split('.');if(parts.length!==3)throw Error('IDENTITY_TOKEN');
 const decode=(s:string)=>JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(base64url(s)));
 const header=headerSchema.parse(decode(parts[0]));if(!discovery.metadata.id_token_signing_alg_values_supported.includes(header.alg))throw Error('IDENTITY_ALGORITHM');
 const matching=discovery.jwks.keys.filter(k=>k.kid===header.kid);if(matching.length!==1)throw Error('IDENTITY_KEY');const jwk=matching[0];
 if(jwk.kty!=='RSA'||typeof jwk.n!=='string'||base64url(jwk.n).length<256||base64url(jwk.n).length>1024||jwk.alg!==undefined&&jwk.alg!=='RS256'||jwk.use!==undefined&&jwk.use!=='sig'||['d','p','q','dp','dq','qi','oth','k'].some(k=>k in jwk))throw Error('IDENTITY_KEY');
 const algorithm={name:'RSASSA-PKCS1-v1_5',hash:'SHA-256'};const key=await crypto.subtle.importKey('jwk',jwk as unknown as JsonWebKey,algorithm,false,['verify']);
 if(!await crypto.subtle.verify(algorithm,key,base64url(parts[2]),encoder.encode(parts[0]+'.'+parts[1])))throw Error('IDENTITY_SIGNATURE');
 const claims=claimsSchema.parse(decode(parts[1])),audiences=typeof claims.aud==='string'?[claims.aud]:claims.aud;
 if(claims.iss!==config.issuer||!audiences.includes(audience)||claims.azp!==undefined&&claims.azp!==audience||audiences.length>1&&claims.azp!==audience||claims.exp<=now||claims.iat>now+30||claims.exp<=claims.iat||claims.exp-claims.iat>7200||claims.nbf!==undefined&&claims.nbf>now)throw Error('IDENTITY_CLAIMS');
 return claims;
}
export async function exchangeCode(env:Env,config:IdentityConfig,discovery:Discovery,code:string,verifier:string,transport:typeof fetch=fetch):Promise<string>{
 if(!code||code.length>4096||!/^[A-Za-z0-9_-]{43}$/.test(verifier))throw Error('IDENTITY_CODE');
 const body=new URLSearchParams({grant_type:'authorization_code',code,code_verifier:verifier,redirect_uri:config.webRedirect,client_id:config.webClientId,client_secret:env.OIDC_WEB_CLIENT_SECRET!});
 const response=await transport(discovery.metadata.token_endpoint,{method:'POST',body,headers:{'Content-Type':'application/x-www-form-urlencoded',Accept:'application/json'},redirect:'manual',signal:AbortSignal.timeout(10000)});
 if(!response.ok)throw Error('IDENTITY_EXCHANGE');const result=z.object({id_token:z.string().min(1).max(16384),token_type:z.string().optional()}).parse(JSON.parse(new TextDecoder().decode(await readBounded(response.body,65536))));return result.id_token;
}
export async function resolveUser(env:Env,issuer:string,subject:string):Promise<string>{
 const now=Date.now(),candidate=crypto.randomUUID();
 // D1 batch is transactional; INSERT..SELECT and the following identity insert serialize races.
 await env.DB.batch([env.DB.prepare('INSERT INTO users(id,created_at) SELECT ?,? WHERE NOT EXISTS(SELECT 1 FROM external_identities WHERE issuer=? AND subject=?)').bind(candidate,now,issuer,subject),env.DB.prepare('INSERT INTO external_identities(issuer,subject,user_id,created_at) SELECT ?,?,?,? WHERE EXISTS(SELECT 1 FROM users WHERE id=?) ON CONFLICT(issuer,subject) DO NOTHING').bind(issuer,subject,candidate,now,candidate)]);
 const row=await env.DB.prepare('SELECT user_id FROM external_identities WHERE issuer=? AND subject=?').bind(issuer,subject).first<{user_id:string}>();if(!row)throw Error('IDENTITY_USER');return row.user_id;
}
export async function desktopIdentity(request:Request,env:Env,config:IdentityConfig,transport:typeof fetch=fetch){
 const auth=request.headers.get('Authorization');if(!auth||auth.length>16400||!auth.startsWith('Bearer '))throw Error('IDENTITY_BEARER');
 const discovery=await discoverIdentity(config,env.KORTEX_ENV,transport),claims=await verifyIdToken(auth.slice(7),config,discovery,config.desktopClientId);
 // Google ID tokens establish identity only. They are NEVER Kortex API write/sync grants.
 const identity=await env.DB.prepare('SELECT user_id FROM external_identities WHERE issuer=? AND subject=?').bind(claims.iss,claims.sub).first<{user_id:string}>();if(!identity)throw Error('IDENTITY_USER');
 return {userId:identity.user_id,scopes:['identity:read'] as const};
}
