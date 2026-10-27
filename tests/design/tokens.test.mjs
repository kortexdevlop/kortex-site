import {test} from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {renderTokens} from '../../scripts/build-design-tokens.mjs';
const tokens = JSON.parse(readFileSync(new URL('../../website/src/styles/tokens.json', import.meta.url), 'utf8'));
const luminance = hex => {
  const rgb = hex.slice(1).match(/../g).map(x => parseInt(x,16)/255).map(x => x <= .04045 ? x/12.92 : ((x+.055)/1.055)**2.4);
  return rgb[0]*.2126 + rgb[1]*.7152 + rgb[2]*.0722;
};
const contrast = (a,b) => { const x=luminance(a),y=luminance(b); return (Math.max(x,y)+.05)/(Math.min(x,y)+.05); };
test('normal text and muted text meet AA on every surface in both themes', () => {
  for (const [name, colors] of Object.entries(tokens.themes)) {
    for (const background of ['surface','raised',...Object.keys(colors).filter(x=>x.startsWith('card-'))]) {
      for (const text of ['foreground','muted']) assert.ok(contrast(colors[text],colors[background]) >= 4.5, `${name} ${text}/${background}: ${contrast(colors[text],colors[background])}`);
    }
    assert.ok(contrast(colors['on-accent'],colors.accent) >= 4.5, `${name} button`);
    for (const background of ['surface','raised']) {
      assert.ok(contrast(colors.focus,colors[background]) >= 3, `${name} focus`);
      assert.ok(contrast(colors['control-border'],colors[background]) >= 3, `${name} control outline`);
      assert.ok(contrast(colors.accent,colors[background]) >= 4.5, `${name} accent text`);
    }
  }
});
test('generated CSS is reproducible and source values do not diverge', () => {
  const generated = readFileSync(new URL('../../website/src/styles/theme.css',import.meta.url),'utf8').replaceAll('\r\n','\n');
  assert.equal(generated,renderTokens(tokens));
  const global=readFileSync(new URL('../../website/src/styles/global.css',import.meta.url),'utf8');
  assert.doesNotMatch(global, /#[0-9a-f]{3,8}\b|\brgba?\(|\bhsla?\(/i);
  for (const values of Object.values(tokens.spacing)) assert.equal(parseInt(values)%4,0);
  assert.deepEqual(Object.keys(tokens.densities),['simple','medium','pro']);
  assert.deepEqual(Object.keys(tokens.themes.light),Object.keys(tokens.themes.dark));
});
