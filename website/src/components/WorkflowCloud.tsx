import {useState} from 'preact/hooks';
import {accountRequest,accountSession,accountHeaders,AccountError} from '../lib/account-client';
import {persistentWorkflow} from '../lib/workflow-storage';
import {serialize,type Workflow} from '../lib/workflows';
type Row={id:string;workflow:Workflow|null;revision:number};
export default function WorkflowCloud({workflow,onLoad}:{workflow:Workflow;onLoad:(json:string)=>void}){
 const [rows,setRows]=useState<Row[]>([]),[etag,setEtag]=useState(''),[notice,setNotice]=useState(''),[busy,setBusy]=useState(false),[owner,setOwner]=useState('');
 async function action(kind:'read'|'write'|'delete'){
  if(busy)return;setBusy(true);setNotice('');try{const session=await accountSession();if(!session.user.sync_enabled)throw Error('Activez d’abord la synchronisation dans votre compte.');if(kind!=='read'&&(owner!==session.user.id||!etag))throw Error('Rechargez vos parcours avant de les modifier.');
   const response=await accountRequest('/api/v1/me/workflows'+(kind==='read'?'':'/'+encodeURIComponent(workflow.id)),kind==='read'?{}:{method:kind==='write'?'PUT':'DELETE',headers:accountHeaders(session,etag),...(kind==='write'?{body:JSON.stringify(persistentWorkflow(workflow))}:{})});
   if(kind==='read'){const data=await response.json() as {items:Row[]};setRows(data.items);setEtag(response.headers.get('ETag')??'');setOwner(session.user.id);setNotice('Parcours du compte chargés. Aucun résultat d’exécution envoyé.')}else{setEtag('');setRows([]);setNotice(kind==='write'?'Parcours sauvegardé dans votre compte. Rechargez la liste pour continuer.':'Parcours supprimé du compte. La copie locale est conservée.')}
  }catch(e){if(e instanceof AccountError&&e.status===409){setEtag('');setNotice('Une autre copie a changé. Votre travail local est conservé : rechargez la liste et comparez avant de sauvegarder.')}else setNotice(e instanceof AccountError?e.status===401?'Connectez-vous pour utiliser la synchronisation.':'Synchronisation indisponible.':e instanceof Error?e.message:'Synchronisation indisponible.')}finally{setBusy(false)}
 }
 return <details class="wf-exchange"><summary>Synchronisation facultative du compte</summary><p>Seule la définition du parcours est envoyée à votre demande. Les résultats et fichiers restent sur cet appareil.</p><button disabled={busy} onClick={()=>void action('read')}>Charger mes parcours du compte</button><button disabled={busy||!etag} onClick={()=>void action('write')}>Sauvegarder ce parcours dans mon compte</button><button disabled={busy||!etag||!rows.some(r=>r.id===workflow.id&&r.workflow)} onClick={()=>void action('delete')}>Supprimer ce parcours du compte</button><ul>{rows.filter(r=>r.workflow).map(r=><li><button disabled={busy} onClick={()=>{onLoad(serialize(persistentWorkflow(r.workflow)));setNotice('Copie transmise à l’éditeur. Les actions publiées sont revérifiées.')}}>{r.workflow!.name} · révision {r.revision}</button></li>)}</ul><p role="status">{notice}</p></details>
}
