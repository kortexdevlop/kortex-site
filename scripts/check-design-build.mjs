import { readdirSync, readFileSync } from 'node:fs';
const root = new URL('../website/dist/', import.meta.url);
const entries = readdirSync(root, {recursive: true}).map(String);
if (entries.some(path => /__dev|design\.html/.test(path))) throw Error('Development route leaked into static output');
for (const entry of entries.filter(path => path.endsWith('.html'))) {
  if (readFileSync(new URL(entry.replaceAll('\\','/'), root),'utf8').includes('Atelier local')) throw Error('Development content leaked into HTML');
}
console.log('Static output: development preview absent');
