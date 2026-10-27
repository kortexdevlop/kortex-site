import {test,expect} from 'playwright/test';
test('XML valide, uniquement les outils publiés, index et robots',async({page,request})=>{
 const index=await request.get('/sitemap-index.xml');expect(index.status()).toBe(200);expect(index.headers()['cache-control']).toBe('public, max-age=900');
 await page.goto('/');const parse=(xml:string)=>page.evaluate(xml=>{const document=new DOMParser().parseFromString(xml,'application/xml');return {errors:document.querySelectorAll('parsererror').length,locations:[...document.querySelectorAll('loc')].map(el=>el.textContent),dates:[...document.querySelectorAll('lastmod')].map(el=>el.textContent)};},xml);
 const sitemap=await parse(await index.text());expect(sitemap.errors).toBe(0);expect(sitemap.locations.length).toBeGreaterThan(0);const urls:string[]=[];
 for(const path of sitemap.locations){const r=await request.get(path!);expect(r.status()).toBe(200);const value=await parse(await r.text());expect(value.errors).toBe(0);urls.push(...value.locations as string[]);expect(value.dates.every(d=>!Number.isNaN(Date.parse(d!)))).toBe(true);}
 expect(new Set(urls).size).toBe(urls.length);expect(urls).toContain('http://127.0.0.1:4321/tools/calculatrice');expect(urls.some(u=>/outil-cache|\/app\//.test(u))).toBe(false);
 expect((await request.get('/sitemaps/-1.xml')).status()).toBe(404);expect((await request.get('/sitemaps/99999.xml')).status()).toBe(404);expect((await request.get('/api/v1/internal/sitemap?page=0')).status()).toBe(404);
 const robots=await request.get('/robots.txt');expect(await robots.text()).toContain('Disallow: /app/');
 await page.goto('/tools/calculatrice');await expect(page.locator('meta[property="og:title"]')).toHaveAttribute('content',/Calculatrice/);const json=JSON.parse(await page.locator('script[type="application/ld+json"]').textContent()||'{}');expect(json['@type']).toBe('SoftwareApplication');expect(json.aggregateRating).toBeUndefined();
 const app=await request.get('/app/calculatrice');expect(app.status()).toBe(200);for(const match of (await app.text()).matchAll(/<script([^>]*)>[\s\S]*?<\/script>/g))expect(match[1]).toContain('src=');
});
