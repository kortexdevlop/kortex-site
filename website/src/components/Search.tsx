import {useEffect,useRef,useState} from 'preact/hooks';
import {ApiClient} from '../lib/api-client';
import {palette,toolHref,type ToolSummary} from '../lib/tools';
interface Props {demoTools?:ToolSummary[];}
export default function Search({demoTools}:Props) {
  const [query,setQuery]=useState('');const [items,setItems]=useState<ToolSummary[]>(demoTools??[]);
  const [status,setStatus]=useState<'idle'|'loading'|'error'>('idle');const serial=useRef(0);
  useEffect(()=>()=>{serial.current++;},[]);
  async function search(event:Event) {
    event.preventDefault();const current=++serial.current;setStatus('loading');
    try {
      const result=demoTools ? demoTools.filter(t=>(t.name+' '+t.summary+' '+t.category).normalize('NFKC').toLocaleLowerCase('fr').includes(query.normalize('NFKC').trim().toLocaleLowerCase('fr'))) : (await new ApiClient().listTools(new URLSearchParams({q:query}))).items;
      if(current===serial.current){setItems(result);setStatus('idle');}
    }catch{if(current===serial.current){setItems([]);setStatus('error');}}
  }
  return <section class="search-island" aria-label={demoTools?'Recherche dans les cartes de démonstration':'Recherche du catalogue'}>
    <form class="search" onSubmit={search} role="search" aria-label="Filtrer les outils"><label class="sr-only" for="catalog-query">Rechercher dans les outils</label><input id="catalog-query" name="q" type="search" maxLength={200} value={query} onInput={e=>setQuery(e.currentTarget.value)}/><button class="button button-primary" type="submit">Rechercher</button></form>
    <p role={status==='error'?'alert':'status'} aria-live="polite">{status==='error'?'Le service est momentanément indisponible.':status==='loading'?'Chargement…':`${items.length} résultat${items.length===1?'':'s'}`}{demoTools?' · Démonstration locale':''}</p>
    <div class="tool-grid" aria-busy={status==='loading'}>{status==='idle'&&items.map(tool=><article key={tool.id} class={`tool-card pastel-${palette(tool)}`}><a class="tool-card-link" href={toolHref(tool)}><span class="tool-icon" aria-hidden="true">{tool.name.charAt(0)}</span><div><p class="tool-category">{tool.category}</p><h3>{tool.name}</h3><p class="tool-summary">{tool.summary}</p></div><span aria-hidden="true">↗</span></a></article>)}</div>
    {status==='idle'&&items.length===0&&<p>Aucun outil ne correspond à votre recherche.</p>}
  </section>;
}
