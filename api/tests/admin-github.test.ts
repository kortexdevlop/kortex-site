import {it,expect,vi} from 'vitest';
import {authorize} from '../src/security/admin-auth';
import type {Env} from '../src/env';

it('accepts the signed GitHub OIDC certificate thumbprint header and rejects extensions',async()=>{
 const pair=await crypto.subtle.generateKey({name:'RSASSA-PKCS1-v1_5',modulusLength:2048,publicExponent:new Uint8Array([1,0,1]),hash:'SHA-256'},true,['sign','verify']) as CryptoKeyPair;
 const jwk=await crypto.subtle.exportKey('jwk',pair.publicKey),kid='github-header-regression';
 const policy={issuer:'https://token.actions.githubusercontent.com',audience:'kortex',subject:'repo:kortexdevlop@334040072/kortex-site@1437766566:environment:staging',repository:'kortexdevlop/kortex-site',repository_id:'1437766566',ref:'refs/heads/main',workflow_ref:'kortexdevlop/kortex-site/.github/workflows/tools.yml@refs/heads/main',actions:['publish']};
 const config={ADMIN_AUTH_POLICY:JSON.stringify(policy),KORTEX_ENV:'staging'} as Env;
 const enc=new TextEncoder(),b64=(value:Uint8Array)=>btoa(String.fromCharCode(...value)).replaceAll('+','-').replaceAll('/','_').replace(/=+$/,'');
 const now=Math.floor(Date.now()/1000),payload=b64(enc.encode(JSON.stringify({iss:policy.issuer,aud:policy.audience,sub:policy.subject,repository:policy.repository,repository_id:policy.repository_id,ref:policy.ref,workflow_ref:policy.workflow_ref,iat:now,exp:now+300,jti:'github-header-test'})));
 const mock=vi.spyOn(globalThis,'fetch').mockImplementation(async(_input,init)=>{expect(init?.redirect).toBe('manual');return Response.json({keys:[{...jwk,kid,alg:'RS256',use:'sig'}]})});
 try{
  async function request(extra:Record<string,unknown>){const head=b64(enc.encode(JSON.stringify({alg:'RS256',kid,typ:'JWT',...extra}))),message=head+'.'+payload,signature=b64(new Uint8Array(await crypto.subtle.sign('RSASSA-PKCS1-v1_5',pair.privateKey,enc.encode(message))));return new Request('https://api.test',{headers:{Authorization:'Bearer '+message+'.'+signature}})}
  mock.mockResolvedValueOnce(new Response(null,{status:302,headers:{Location:'https://attacker.test/keys'}}));
  await expect(authorize(await request({}),config,'publish')).rejects.toMatchObject({status:401});
  await expect(authorize(await request({x5t:'K89yXf4wC_3kQ0P-rjfDHCnCFJE'}),config,'publish')).resolves.toEqual({subject:policy.subject,jti:'github-header-test'});
  for(const extra of [{x5t:'bad'},{jku:'https://attacker.test/keys'},{crit:['x5t']}])await expect(authorize(await request(extra),config,'publish')).rejects.toMatchObject({status:401});
 }finally{mock.mockRestore()}
});
