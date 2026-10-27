import { readdirSync, readFileSync } from 'node:fs';
const root = new URL('../website/dist/', import.meta.url);
const entries = readdirSync(root, {recursive: true}).map(String);
if (entries.some(path => /__dev|design\.html/.test(path))) throw Error('Development route leaked into static output');
for (const entry of entries.filter(path => /\.(?:html|js|mjs|json)$/.test(path))) {
  const text=readFileSync(new URL(entry.replaceAll('\\','/'), root),'utf8');
  if (/Atelier local|demo-visual-|Neuf cartes, une base commune/.test(text)) throw Error('Development content leaked into output');
}
if(!entries.some(path=>path.replaceAll('\\','/').endsWith('wrangler.json')))throw Error('Worker manifest missing');
if(!entries.some(path=>path.replaceAll('\\','/').endsWith('favicon.svg')))throw Error('Public favicon missing');
console.log('SSR output: Worker manifest and favicon present, development preview absent');
