# Intégration du site SSR

Depuis la racine du dépôt :

```powershell
pnpm --filter @kortex/website build
pnpm exec playwright test --config tests/website.config.ts
```

Préparer auparavant les sorties du Runtime Web et de la calculatrice selon les rapports 5.4 et 8.0 : `target/web-dist/wasm/kortex-runtime-web.js`, ses fichiers WASM/adapter/CSS associés et `target/calculatrice-fixture/module.wasm`. La calculatrice peut être reconstruite avec `node tests/fixtures/prepare-calculatrice.mjs`. La suite signe à nouveau le paquet avec une clé de test éphémère, sans persister de clé privée.

Ports locaux réservés : 4321 (site/API dans un même Wrangler multi-worker), 4331 (site sans API pour les erreurs) et 4340 (vrai Runtime). Ne pas lancer simultanément le serveur Astro de développement sur 4321. Playwright démarre et arrête le banc ; `KORTEX_WEBSITE_REUSE=1` permet un banc lancé manuellement avec `node tests/website/server.mjs`.

Chaque lancement applique les migrations à un nouveau répertoire SQLite sous `target/website-integration/state-*`. Les insertions du test utilisent uniquement ce répertoire local. Les enregistrements de catalogue portent des attestations TEST ; seule la calculatrice est réellement exécutée et son archive vérifiée par le Runtime. Aucun accès à D1/R2 distant, aucune publication.

La régression de la maquette est séparée :

```powershell
$env:KORTEX_DESIGN_REPORT='docs/evidence/website-workspace-regression/playwright.json'
$env:KORTEX_WORKSPACE_EVIDENCE='docs/evidence/website-workspace-regression'
pnpm exec playwright test tests/visual/workspace.spec.ts
```

Elle lance l’atelier Astro et conserve les preuves historiques du lot précédent.
