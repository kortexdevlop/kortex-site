import {z} from 'zod';
import type {Env} from '../env';
import {AdminError,readBounded} from '../lib/admin-error';
export type AdminAction='publish'|'revoke'|'rollback';
const policySchema=z.object({issuer:z.enum(['https://token.actions.githubusercontent.com','urn:kortex:staging-operator']),audience:z.string().min(1).max(256),subject:z.string().min(1).max(512),repository:z.string().min(1).max(256),repository_id:z.string().regex(/^\d+$/),ref:z.string().min(1).max(256),workflow_ref:z.string().min(1).max(512),actions:z.array(z.enum(['publish','revoke','rollback'])).min(1),jwks:z.object({keys:z.array(z.record(z.string(),z.unknown())).max(10)}).optional()}).strict();
const claimsSchema=z.object({iss:z.string(),aud:z.string(),sub:z.string(),repository:z.string(),repository_id:z.string(),ref:z.string(),workflow_ref:z.string(),iat:z.number().int(),nbf:z.number().int().optional(),exp:z.number().int(),jti:z.string().min(1).max(256)});
const headerSchema=z.object({alg:z.enum(['RS256','EdDSA']),kid:z.string().min(1).max(128),typ:z.literal('JWT'),x5t:z.string().regex(/^[A-Za-z0-9_-]{27}$/).optional()}).strict();
export function base64url(text:string):Uint8Array {
 if(!/^[A-Za-z0-9_-]+$/.test(text))throw Error('BAD_ENCODING');const b=Uint8Array.from(atob(text.replaceAll('-','+').replaceAll('_','/')),c=>c.charCodeAt(0));
 if(btoa(String.fromCharCode(...b)).replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'')!==text)throw Error('BAD_ENCODING');return b;
}
let githubKeys:{expires:number;keys:Record<string,unknown>[]} | undefined;
async function keys():Promise<Record<string,unknown>[]> {
 if(githubKeys&&githubKeys.expires>Date.now())return githubKeys.keys;
 const response=await fetch('https://token.actions.githubusercontent.com/.well-known/jwks',{redirect:'error',signal:AbortSignal.timeout(5000)});
 if(!response.ok)throw Error('JWKS_UNAVAILABLE');
 const doc=JSON.parse(new TextDecoder().decode(await readBounded(response.body,65536)));
 if(!Array.isArray(doc.keys)||doc.keys.length>20)throw Error('BAD_JWKS');githubKeys={expires:Date.now()+300000,keys:doc.keys};return githubKeys.keys;
}
export async function authorize(request:Request,env:Env,action:AdminAction):Promise<{subject:string;jti:string}> {
 let policy:z.infer<typeof policySchema>;
 try{policy=policySchema.parse(JSON.parse(env.ADMIN_AUTH_POLICY??''));if(env.KORTEX_ENV==='production'&&policy.issuer!=='https://token.actions.githubusercontent.com')throw Error('OPERATOR_FORBIDDEN')}catch{throw new AdminError(503,'UNAVAILABLE','Admin authorization unavailable')}
 const auth=request.headers.get('Authorization');if(!auth||auth.length>16384||!/^Bearer [A-Za-z0-9_.-]+$/.test(auth))throw new AdminError(401,'UNAUTHORIZED','Admin authorization required');
 try {
  const parts=auth.slice(7).split('.');if(parts.length!==3)throw Error('JWT');
  const header=headerSchema.parse(JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(base64url(parts[0]))));
  const staging=policy.issuer==='urn:kortex:staging-operator';if(staging?header.alg!=='EdDSA':header.alg!=='RS256')throw Error('ALGORITHM');
  const available=staging?policy.jwks?.keys:await keys();const selected=available?.filter(k=>k.kid===header.kid);if(selected?.length!==1)throw Error('KEY');const jwk=selected[0];
  if(staging?(jwk.kty!=='OKP'||jwk.crv!=='Ed25519'):(jwk.kty!=='RSA'||typeof jwk.n!=='string'||base64url(jwk.n).length<256))throw Error('KEY_TYPE');
  if(jwk.d!==undefined||jwk.alg&&jwk.alg!==header.alg||jwk.use&&jwk.use!=='sig')throw Error('KEY_USE');
  const algorithm=staging?{name:'Ed25519'}:{name:'RSASSA-PKCS1-v1_5',hash:'SHA-256'};
  const key=await crypto.subtle.importKey('jwk',jwk as unknown as JsonWebKey,algorithm,false,['verify']);
  if(!await crypto.subtle.verify(algorithm,key,base64url(parts[2]),new TextEncoder().encode(parts[0]+'.'+parts[1])))throw Error('SIGNATURE');
  const c=claimsSchema.parse(JSON.parse(new TextDecoder('utf-8',{fatal:true,ignoreBOM:false}).decode(base64url(parts[1])))),now=Math.floor(Date.now()/1000);
  if(c.iss!==policy.issuer||c.aud!==policy.audience||c.sub!==policy.subject||c.repository!==policy.repository||c.repository_id!==policy.repository_id||c.ref!==policy.ref||c.workflow_ref!==policy.workflow_ref||c.exp<=now||c.exp<=c.iat||c.exp-c.iat>600||c.iat>now+30||c.iat<now-600||c.nbf!==undefined&&c.nbf>now)throw Error('CLAIMS');
  if(!policy.actions.includes(action))throw new AdminError(403,'FORBIDDEN','Admin action forbidden');
  return {subject:c.sub,jti:c.jti};
 }catch(error){if(error instanceof AdminError&&error.status===403)throw error;throw new AdminError(401,'UNAUTHORIZED','Invalid admin authorization')}
}
