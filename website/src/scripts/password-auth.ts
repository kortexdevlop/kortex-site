type Me={user:{id:string;email?:string;email_verified?:boolean};csrf_token?:string};
import {accountReturn} from '../lib/return-to';
async function request(path:string,init:RequestInit={}){return fetch(path,{...init,credentials:'same-origin',cache:'no-store',redirect:'error',signal:AbortSignal.timeout(30000)})}
const panel=document.querySelector<HTMLElement>('[data-password-auth]');
if(panel){
 const form=panel.querySelector<HTMLFormElement>('form')!,submit=form.querySelector<HTMLButtonElement>('[type=submit]')!,status=panel.querySelector<HTMLElement>('[data-auth-status]')!,password=form.querySelector<HTMLInputElement>('[name=password]')!,email=form.querySelector<HTMLInputElement>('[name=email]')!,register=panel.dataset.mode==='register';let csrf='',ready=false;
 panel.querySelector<HTMLButtonElement>('[data-toggle-password]')!.addEventListener('click',event=>{const button=event.currentTarget as HTMLButtonElement,show=password.type==='password';password.type=show?'text':'password';button.textContent=show?'Masquer':'Afficher';button.setAttribute('aria-label',show?'Masquer le mot de passe':'Afficher le mot de passe');button.setAttribute('aria-pressed',String(show))});
 async function prepare(){
  const options=await request('/api/v1/auth/options');if(!options.ok)throw Error('unavailable');const value=await options.json() as {password:boolean;google:boolean};panel!.querySelectorAll<HTMLElement>('[data-google]').forEach(el=>el.hidden=!value.google);if(!value.password)throw Error('unavailable');
  const response=await request('/api/v1/auth/password/token');if(!response.ok)throw Error('unavailable');const body=await response.json() as {csrf_token:string};if(!/^[a-f0-9]{64}$/.test(body.csrf_token))throw Error('unavailable');csrf=body.csrf_token;ready=true;submit.disabled=false;status.textContent='';
 }
 void prepare().catch(()=>{status.textContent='La connexion est momentanément indisponible. Vous pouvez continuer sans compte.'});
 form.addEventListener('submit',async event=>{
  event.preventDefault();if(!ready)return;const confirmation=form.querySelector<HTMLInputElement>('[name=confirmation]');if(confirmation&&confirmation.value!==password.value){status.textContent='Les deux mots de passe ne correspondent pas.';confirmation.focus();return}
  if(register&&([...password.value].length<15||[...password.value].length>128)){status.textContent='Choisissez un mot de passe de 15 à 128 caractères.';password.focus();return}
  submit.disabled=true;status.textContent=register?'Création de votre compte…':'Connexion en cours…';
  try{
   const response=await request('/api/v1/auth/password/'+(register?'register':'login'),{method:'POST',headers:{'Content-Type':'application/json','X-CSRF-Token':csrf},body:JSON.stringify({email:email.value,password:password.value})});
   if(response.ok){password.value='';if(confirmation)confirmation.value='';location.assign(accountReturn(new URLSearchParams(location.search).get('returnTo')));return}
   if(response.status===403){await prepare();status.textContent='Le formulaire a expiré. Réessayez.'}
   else status.textContent=response.status===429?'Trop de tentatives. Patientez avant de réessayer.':response.status===401?'Email ou mot de passe incorrect.':response.status===409?'Impossible de créer ce compte avec ces informations. Essayez de vous connecter.':response.status===400?'Vérifiez votre email et choisissez un mot de passe personnel de 15 à 128 caractères.':'La connexion est momentanément indisponible.';
  }catch{status.textContent='La connexion a été interrompue. Réessayez.'}finally{submit.disabled=false}
 });
}
const account=document.querySelector<HTMLElement>('[data-account]');
if(account){
 const status=account.querySelector<HTMLElement>('[data-auth-status]')!,logout=account.querySelector<HTMLButtonElement>('[data-logout]')!;let csrf='';
 void request('/api/v1/me').then(async response=>{if(response.status===401||response.status===404){location.replace('/auth/login');return}if(!response.ok)throw Error('unavailable');const body=await response.json() as Me;if(!body.user?.id||!body.csrf_token)throw Error('unavailable');csrf=body.csrf_token;account.querySelector<HTMLElement>('[data-account-email]')!.textContent=body.user.email??'Connecté avec Google';account.querySelector<HTMLElement>('[data-email-verification]')!.hidden=!body.user.email||!!body.user.email_verified;logout.disabled=false}).catch(()=>status.textContent='Votre compte est momentanément indisponible.');
 logout.addEventListener('click',async()=>{logout.disabled=true;try{const response=await request('/api/v1/auth/logout',{method:'POST',headers:{'X-CSRF-Token':csrf}});if(!response.ok)throw Error('unavailable');location.replace('/auth/login')}catch{status.textContent='La déconnexion a échoué. Réessayez.';logout.disabled=false}});
}
