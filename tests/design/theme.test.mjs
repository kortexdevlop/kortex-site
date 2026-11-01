import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync,readdirSync} from 'node:fs';
const styles = new URL('../../website/src/styles/',import.meta.url);
const read = path => readFileSync(new URL(path,import.meta.url),'utf8');
test('stylesheets take every colour from the theme tokens', () => {
  const generated = new Set(['theme.css','workspace.css']);
  for (const name of readdirSync(styles).filter(file => file.endsWith('.css') && !generated.has(file))) {
    assert.doesNotMatch(readFileSync(new URL(name,styles),'utf8').replaceAll(/%23[0-9a-f]{3,8}/gi,''), /#[0-9a-f]{3,8}\b|\brgba\(|\bhsla?\(/i, name);
  }
  assert.doesNotMatch(read('../../website/src/layouts/LegalLayout.astro'), /#[0-9a-f]{3,8}\b|\brgba\(/i);
});
test('no page forces a theme and every layout applies the saved one', () => {
  for (const path of ['layouts/AuthLayout.astro','layouts/LegalLayout.astro','layouts/WorkspaceLayout.astro','pages/index.astro']) {
    const page = read(`../../website/src/${path}`);
    assert.doesNotMatch(page, /<html[^>]*data-theme/, path);
    assert.match(page, /<script is:inline src="\/theme\.js">/, path);
  }
  assert.doesNotMatch(readFileSync(new URL('workspace.source.css',styles),'utf8'), /color-scheme/);
});
