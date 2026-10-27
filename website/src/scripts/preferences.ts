import tokens from '../styles/tokens.json';
const root = document.documentElement;
const words = {
  fr: { tools:'Outils', apps:'Applications', download:'Télécharger', account:'Compte', search:'Rechercher un outil', submit:'Rechercher', menu:'Menu', nav:'Navigation principale', preferences:'Préférences d’affichage', density:'Densité', simple:'Simple', medium:'Moyenne', pro:'Compacte', dark:'Thème sombre', type:'Agrandir le texte', language:'Passer en anglais', footer:'Vos outils, à votre rythme.', top:'Retour en haut ↑', skip:'Aller au contenu' },
  en: { tools:'Tools', apps:'Applications', download:'Download', account:'Account', search:'Search for a tool', submit:'Search', menu:'Menu', nav:'Main navigation', preferences:'Display preferences', density:'Density', simple:'Relaxed', medium:'Medium', pro:'Compact', dark:'Dark theme', type:'Larger text', language:'Switch to French', footer:'Your tools, at your pace.', top:'Back to top ↑', skip:'Skip to content' },
};
type Language = keyof typeof words;
type Word = keyof typeof words.fr;
const theme = document.querySelector<HTMLButtonElement>('#theme-toggle')!;
const type = document.querySelector<HTMLButtonElement>('#type-toggle')!;
const language = document.querySelector<HTMLButtonElement>('#language-toggle')!;
const density = document.querySelector<HTMLSelectElement>('#density')!;
const menu = document.querySelector<HTMLButtonElement>('#menu-toggle')!;
const panel = document.querySelector<HTMLElement>('#header-panel')!;
const mobile = matchMedia(`(width < ${tokens.breakpoint.desktop})`);
const systemDark = matchMedia('(prefers-color-scheme: dark)');
function save() {
  try { localStorage.setItem('kortex-display', JSON.stringify({theme:root.dataset.theme, density:root.dataset.density, type:root.dataset.type, lang:root.lang})); } catch { /* Preferences still work when storage is unavailable. */ }
}
function refresh() {
  const lang: Language = root.lang === 'en' ? 'en' : 'fr';
  for (const [attribute, target] of [['data-i18n','text'], ['data-i18n-label','aria-label'], ['data-i18n-placeholder','placeholder']]) {
    document.querySelectorAll<HTMLElement>(`[${attribute}]`).forEach(element => {
      const key = element.getAttribute(attribute) as Word;
      if (!(key in words[lang])) return;
      if (target === 'text') element.textContent = words[lang][key];
      else element.setAttribute(target, words[lang][key]);
    });
  }
  language.textContent = lang.toUpperCase();
  theme.setAttribute('aria-pressed', String(root.dataset.theme === 'dark' || (!root.dataset.theme && systemDark.matches)));
  type.setAttribute('aria-pressed', String(root.dataset.type === 'large'));
  density.value = root.dataset.density || 'medium';
}
function closeMenu(restore = false) {
  menu.setAttribute('aria-expanded', 'false');
  panel.hidden = mobile.matches;
  if (restore) menu.focus();
}
function resize() {
  const focusInside = panel.contains(document.activeElement);
  menu.hidden = !mobile.matches;
  closeMenu(mobile.matches && focusInside);
}
menu.addEventListener('click', () => {
  const open = menu.getAttribute('aria-expanded') !== 'true';
  menu.setAttribute('aria-expanded', String(open));
  panel.hidden = !open;
});
document.addEventListener('keydown', event => {
  if (event.key === 'Escape' && mobile.matches && !panel.hidden) closeMenu(true);
});
document.addEventListener('click', event => {
  if (mobile.matches && !panel.hidden && event.target instanceof Node && !panel.contains(event.target) && !menu.contains(event.target)) closeMenu();
});
panel.querySelectorAll('nav a').forEach(link => link.addEventListener('click', () => { if (mobile.matches) closeMenu(true); }));
theme.addEventListener('click', () => { root.dataset.theme = theme.getAttribute('aria-pressed') === 'true' ? 'light' : 'dark'; save(); refresh(); });
type.addEventListener('click', () => { root.dataset.type = root.dataset.type === 'large' ? 'standard' : 'large'; save(); refresh(); });
density.addEventListener('change', () => { root.dataset.density = density.value; save(); });
language.addEventListener('click', () => { root.lang = root.lang === 'en' ? 'fr' : 'en'; save(); refresh(); });
mobile.addEventListener('change', resize);
systemDark.addEventListener('change', refresh);
refresh(); resize();
