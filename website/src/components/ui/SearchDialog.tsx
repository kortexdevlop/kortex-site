import {useEffect,useLayoutEffect,useRef,useState} from 'preact/hooks';
import {liveSearch} from '../../lib/search';
import {accessibility} from '../../lib/accessibility';
export default function SearchDialog(){
 const dialog=useRef<HTMLDialogElement>(null),input=useRef<HTMLInputElement>(null),[state,setState]=useState('ready'),[items,setItems]=useState<{id:string;name:string}[]>([]),[language,setLanguage]=useState<'fr'|'en'>('fr'),[large,setLarge]=useState(false);const english=language==='en';
 useEffect(()=>{try{const saved=JSON.parse(localStorage.getItem('kortex-accessibility')||'{}');if(saved.language==='en')setLanguage('en');setLarge(saved.large===true);}catch{}},[]);
 useEffect(()=>accessibility(language,large),[language,large]);
 useLayoutEffect(()=>{const global=document.querySelector<HTMLInputElement>('#ws-global-query');let restoring=false;
 const open=()=>{if(restoring||dialog.current!.open)return;dialog.current!.showModal();input.current!.focus();};const close=()=>{restoring=true;global?.focus();queueMicrotask(()=>restoring=false);};
 const key=(event:KeyboardEvent)=>{if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();open();}};window.addEventListener('keydown',key);global?.addEventListener('click',open);dialog.current!.addEventListener('close',close);
 const stop=liveSearch(input.current!,(state,items)=>{setState(state);setItems(items||[]);});dialog.current!.dataset.ready='true';if(document.activeElement===global||new URLSearchParams(location.search).has('search'))open();return ()=>{stop();window.removeEventListener('keydown',key);global?.removeEventListener('click',open);dialog.current?.removeEventListener('close',close);};},[]);
 return <dialog class="ws-search-dialog" ref={dialog} data-search-dialog aria-labelledby="ws-search-title">
  <button class="ws-dialog-close" type="button" onClick={()=>dialog.current!.close()} aria-label={english?'Close':'Fermer'}>×</button>
  <h2 id="ws-search-title">{english?'Find a tool':'Rechercher un outil'}</h2>
  <form action="/tools"><label for="ws-live-query">{english?'Search the catalogue':'Rechercher dans le catalogue'}</label><input ref={input} id="ws-live-query" type="search" name="q" maxLength={100} autoComplete="off" onKeyDown={event=>{if(event.key==='ArrowDown'){event.preventDefault();dialog.current!.querySelector<HTMLAnchorElement>('[data-search-result]')?.focus();}}}/><button type="submit">{english?'All results':'Tous les résultats'}</button></form>
  <p role="status">{state==='loading'?(english?'Searching…':'Recherche en cours…'):state==='error'?(english?'Search unavailable. Try again.':'Recherche indisponible. Réessayez.'):(input.current?.value&&!items.length?(english?'No results':'Aucun résultat'):'')}</p>
  <nav aria-label={english?'Search results':'Résultats de recherche'}><ul>{items.map(item=><li key={item.id}><a data-search-result href={'/tools/'+encodeURIComponent(item.id)}>{item.name}</a></li>)}</ul></nav>
  <fieldset><legend>{english?'Display preferences':'Préférences d’affichage'}</legend><label>{english?'Language':'Langue'} <select value={language} onChange={e=>setLanguage(e.currentTarget.value as 'fr'|'en')}><option value="fr">Français</option><option value="en">English</option></select></label><button type="button" aria-pressed={large} onClick={()=>setLarge(!large)}>{english?'Aa — Larger text':'Aa — Agrandir le texte'}</button></fieldset>
 </dialog>;
}
