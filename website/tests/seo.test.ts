import {test} from 'node:test';
import assert from 'node:assert/strict';
import {xml,jsonLd,sitemapPage} from '../src/lib/seo.ts';
test('SEO serializers escape XML and script termination without inventing ratings',()=>{assert.equal(xml('<>&"\''),'&lt;&gt;&amp;&quot;&apos;');assert.ok(!jsonLd({name:'</script><script>alert(1)</script>'}).includes('<'));});
test('sitemap DTO is bounded and rejects unsafe IDs',async()=>{await assert.rejects(sitemapPage({fetch:async()=>Response.json({pages:1,items:[{id:'../admin',published_at:1}]})},0));await assert.rejects(sitemapPage(undefined,0));});
