/* Applies the saved theme before first paint and drives every [data-theme-toggle].
   Served as a file because strict pages only allow scripts from 'self'. */
(() => {
  const root = document.documentElement, key = 'kortex-display';
  const system = matchMedia('(prefers-color-scheme: dark)');
  const read = () => { try { return JSON.parse(localStorage.getItem(key) || '{}'); } catch { return {}; } };
  const saved = read().theme;
  if (saved === 'light' || saved === 'dark') root.dataset.theme = saved;
  const dark = () => root.dataset.theme ? root.dataset.theme === 'dark' : system.matches;
  const sync = () => document.querySelectorAll('[data-theme-toggle]').forEach(button => button.setAttribute('aria-pressed', String(dark())));
  document.addEventListener('DOMContentLoaded', sync);
  system.addEventListener('change', sync);
  document.addEventListener('click', event => {
    if (!(event.target instanceof Element) || !event.target.closest('[data-theme-toggle]')) return;
    const theme = dark() ? 'light' : 'dark';
    root.dataset.theme = theme;
    try { localStorage.setItem(key, JSON.stringify({ ...read(), theme })); } catch { /* The theme still applies to this page. */ }
    sync();
  });
})();
