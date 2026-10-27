export {};
const root=document.querySelector<HTMLElement>('.workspace')!;
const demo=root.dataset.demo==='true';
root.querySelectorAll<HTMLElement>('a,button').forEach(el=>el.tabIndex=0);
const cards=[...root.querySelectorAll<HTMLElement>('.ws-tool-card')];
const main=root.querySelector<HTMLElement>('.ws-main-panel')!;
const scroller=root.querySelector<HTMLElement>('.ws-scroll')!;
const task=root.querySelector<HTMLTextAreaElement>('#ws-task')!;
const query=root.querySelector<HTMLInputElement>('#ws-global-query')!;
const composer=root.querySelector<HTMLFormElement>('.ws-composer')!;
const file=root.querySelector<HTMLInputElement>('#ws-file')!;
const status=root.querySelector<HTMLElement>('.ws-result-status')!;
const dialog=root.querySelector<HTMLDialogElement>('#ws-dialog')!;
const sidebar=root.querySelector<HTMLElement>('.ws-sidebar')!;
const menu=root.querySelector<HTMLButtonElement>('.ws-mobile-menu')!;
const backdrop=root.querySelector<HTMLElement>('.ws-drawer-backdrop')!;
let section='home',category='',filter='',mode='all';
const storageKey=demo?'kortex-workspace-demo-favorites':'kortex-workspace-favorites';
function normalize(s:string){return s.normalize('NFD').replace(/[\u0300-\u036f]/g,'').toLocaleLowerCase('fr');}
function mobile(){return getComputedStyle(document.documentElement).getPropertyValue('--ws-mobile').trim()==='1';}
function countFavorites(){root.querySelector('[data-count=favorites]')!.textContent=String(cards.filter(c=>c.dataset.favorite==='true').length);}
function persistFavorites(){try{localStorage.setItem(storageKey,JSON.stringify(cards.filter(c=>c.dataset.favorite==='true').map(c=>c.dataset.id)));}catch{}}
try{const saved:unknown=JSON.parse(localStorage.getItem(storageKey)||'null');if(Array.isArray(saved)&&saved.every(x=>typeof x==='string'))cards.forEach(c=>{c.dataset.favorite=String(saved.includes(c.dataset.id!));c.querySelector('.ws-favorite')!.setAttribute('aria-pressed',c.dataset.favorite);});}catch{}
function render(announce=true){
  const words=normalize(filter).split(/\W+/).filter(w=>w.length>2&&!['les','des','une','pour','avec','vos','votre','creer','faire','vers','fichier','formater','compresser'].includes(w));
  let visible=0;
  for(const card of cards){const text=normalize(card.textContent||'');const matchesText=!filter.trim()||text.includes(normalize(filter))||words.some(w=>text.includes(w));const show=matchesText&&(!category||card.dataset.toolCategory===category)&&(section!=='favorites'||card.dataset.favorite==='true')&&(section!=='installed'||card.dataset.installed==='true')&&(mode==='all'||card.dataset.toolMode===mode||card.dataset.toolMode==='both');card.hidden=!show;if(show)visible++;}
  root.querySelector('#ws-visible-count')!.textContent=`${visible} outil${visible===1?'':'s'}`;
  root.querySelector<HTMLElement>('.ws-no-results')!.hidden=visible!==0||Boolean(root.querySelector('.ws-api-error'));
  if(announce){status.hidden=false;status.textContent=`${visible} outil${visible===1?'':'s'} ${visible===1?'correspond':'correspondent'} à votre recherche.`;}
  countFavorites();
}
function closeMenu(restore=false){sidebar.classList.remove('is-open');menu.setAttribute('aria-expanded','false');backdrop.hidden=true;main.inert=false;if(restore)menu.focus();}
function reset(){section='home';category='';filter='';mode='all';task.value='';query.value='';file.value='';composer.classList.remove('has-file');root.querySelector<HTMLElement>('.ws-file-status')!.hidden=true;root.querySelector<HTMLInputElement>('input[name=mode][value=all]')!.checked=true;root.querySelectorAll<HTMLElement>('[data-section]').forEach(b=>{b.classList.toggle('is-active',b.dataset.section==='home');if(b.dataset.section==='home')b.setAttribute('aria-current','page');else b.removeAttribute('aria-current');});root.querySelectorAll('[data-category]').forEach(b=>b.removeAttribute('aria-pressed'));root.querySelector('#ws-page-name')!.textContent='Accueil';render(false);status.hidden=true;scroller.scrollTop=0;closeMenu();task.focus();}
root.querySelector('[data-new-task]')!.addEventListener('click',reset);
root.querySelectorAll<HTMLButtonElement>('[data-section]').forEach(button=>button.addEventListener('click',()=>{section=button.dataset.section!;category='';root.querySelectorAll<HTMLElement>('[data-section]').forEach(b=>{b.classList.toggle('is-active',b===button);if(b===button)b.setAttribute('aria-current','page');else b.removeAttribute('aria-current');});root.querySelectorAll('[data-category]').forEach(b=>b.removeAttribute('aria-pressed'));root.querySelector('#ws-page-name')!.textContent=button.querySelector('span:nth-child(2)')!.textContent;render();closeMenu();if(section==='home')scroller.scrollTop=0;else root.querySelector('#workspace-tools')!.scrollIntoView({block:'start'});}));
root.querySelectorAll<HTMLButtonElement>('[data-category]').forEach(button=>button.addEventListener('click',()=>{category=button.dataset.category!;section='all';root.querySelectorAll('[data-category]').forEach(b=>b.setAttribute('aria-pressed',String(b===button)));root.querySelectorAll('[data-section]').forEach(b=>{b.classList.remove('is-active');b.removeAttribute('aria-current');});render();closeMenu();root.querySelector('#workspace-tools')!.scrollIntoView({block:'start'});}));
root.querySelectorAll<HTMLInputElement>('input[name=mode]').forEach(input=>input.addEventListener('change',()=>{mode=input.value;render();}));
query.addEventListener('input',()=>{filter=query.value;render();});
root.querySelector('.ws-global-search')!.addEventListener('submit',event=>{event.preventDefault();filter=query.value;render();root.querySelector('#workspace-tools')!.scrollIntoView({block:'start'});});
composer.addEventListener('submit',event=>{event.preventDefault();filter=task.value.trim();query.value='';render();root.querySelector('#workspace-tools')!.scrollIntoView({block:'start'});});
root.querySelectorAll<HTMLButtonElement>('[data-suggestion]').forEach(button=>button.addEventListener('click',()=>{task.value=button.dataset.suggestion!;composer.requestSubmit();}));
root.querySelectorAll<HTMLButtonElement>('[data-view]').forEach(button=>button.addEventListener('click',()=>{root.querySelector('.ws-tool-grid')!.setAttribute('data-layout',button.dataset.view!);root.querySelectorAll('[data-view]').forEach(b=>b.setAttribute('aria-pressed',String(b===button)));}));
cards.forEach(card=>card.querySelector('.ws-favorite')!.addEventListener('click',()=>{card.dataset.favorite=String(card.dataset.favorite!=='true');card.querySelector('.ws-favorite')!.setAttribute('aria-pressed',card.dataset.favorite);persistFavorites();render(false);}));
function openDialog(title:string,text:string){dialog.querySelector('h2')!.textContent=title;dialog.querySelector('p')!.textContent=text;dialog.showModal();}
root.querySelectorAll<HTMLButtonElement>('.ws-tool-open').forEach(button=>button.addEventListener('click',()=>openDialog(button.dataset.toolTitle!,`${button.dataset.toolSummary} ${demo?'Aperçu visuel : ces outils de démonstration ne sont pas exécutables.':'L’ouverture de cet outil depuis cet espace sera disponible avec le catalogue.'}`)));
root.querySelector('[data-open-account]')!.addEventListener('click',()=>openDialog('Mon espace',demo?'Aperçu du compte de la capture de référence.':'Vous utilisez Kortex sans connexion. Vos favoris sont mémorisés sur cet appareil.'));
root.querySelectorAll('[data-open-docs]').forEach(button=>button.addEventListener('click',()=>openDialog('Votre espace Kortex','Recherchez un outil, choisissez une catégorie ou décrivez votre besoin. Les favoris sont mémorisés sur cet appareil. Les fichiers déposés restent locaux : aucun envoi ou traitement ne démarre automatiquement.')));
root.querySelector('[data-show-recent]')!.addEventListener('click',()=>openDialog('Activités récentes',demo?'Les trois activités affichées reprennent la capture de référence. Il s’agit uniquement de données de démonstration.':'Vos activités récentes s’afficheront après utilisation de vos outils.'));
root.querySelectorAll<HTMLButtonElement>('[data-recent]').forEach(button=>button.addEventListener('click',()=>openDialog(button.dataset.recent!,'Activité de démonstration issue de la référence visuelle. Aucun fichier de votre appareil n’est ouvert.')));
function attachment(selected:File|undefined){if(!selected)return;composer.classList.add('has-file');const output=root.querySelector<HTMLElement>('.ws-file-status')!;output.textContent=`Fichier ajouté localement : ${selected.name}`;output.hidden=false;}
root.querySelector('.ws-attach')!.addEventListener('click',()=>file.click());file.addEventListener('change',()=>attachment(file.files?.[0]));
composer.addEventListener('dragover',event=>{event.preventDefault();composer.classList.add('is-dragging');});composer.addEventListener('dragleave',()=>composer.classList.remove('is-dragging'));composer.addEventListener('drop',event=>{event.preventDefault();composer.classList.remove('is-dragging');attachment(event.dataTransfer?.files[0]);});
menu.addEventListener('click',()=>{const open=menu.getAttribute('aria-expanded')!=='true';if(!open){closeMenu(true);return;}sidebar.classList.add('is-open');menu.setAttribute('aria-expanded','true');backdrop.hidden=false;main.inert=true;sidebar.querySelector<HTMLAnchorElement>('a')!.focus();});
backdrop.addEventListener('click',()=>closeMenu(true));
document.addEventListener('keydown',event=>{if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='k'){event.preventDefault();query.focus();}if((event.ctrlKey||event.metaKey)&&event.key.toLowerCase()==='n'){event.preventDefault();reset();}if(event.key==='Escape'&&sidebar.classList.contains('is-open'))closeMenu(true);if(event.key==='Tab'&&mobile()&&sidebar.classList.contains('is-open')){const elements=[...sidebar.querySelectorAll<HTMLElement>('a,button,input')].filter(el=>!el.hidden);const first=elements[0],last=elements.at(-1)!;if(event.shiftKey&&document.activeElement===first){event.preventDefault();last.focus();}else if(!event.shiftKey&&document.activeElement===last){event.preventDefault();first.focus();}}});
window.addEventListener('resize',()=>{if(!mobile())closeMenu();});
countFavorites();
