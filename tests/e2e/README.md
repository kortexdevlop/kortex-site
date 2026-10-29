# Exécution des gates Kortex

Périmètre autorisé : Windows x64 et Chromium/WebView2. Installer les versions verrouillées indiquées dans `docs/reports/QA_BASELINE.md`, puis `pnpm install --frozen-lockfile`. Les recettes Desktop nécessitent un Windows interactif ; la CI d’installation utilise Windows Server 2022. Aucun résultat macOS/Linux/Firefox n’est revendiqué.

- `node --test tests/release/failure-injection.test.mjs` : reprise, sauvegarde obligatoire, journal et révocations signées, sans accès cloud.
- `node scripts/ci-verify.mjs --lane contracts` : registre, environnement, contrats et sécurisation des publishers.
- `node scripts/ci-verify.mjs --lane web` : API, types, build et sécurité Chromium ; prépare ses fixtures locales.
- `node scripts/ci-verify.mjs --lane native` : format, analyse Rust et tests Wasmtime.
- `node scripts/ci-verify.mjs --lane security` : collecte du gate sécurité.
- `node scripts/qa-isolated.mjs` : drill distant release puis 10k catalogue/SSR/sitemaps et ajout sans rebuild. Nécessite `CLOUDFLARE_API_TOKEN` fourni par l’environnement, accès création/suppression D1/R2/Workers et build Astro existant. Aucun identifiant en argument. Ne pas lancer les anciens scripts de benchmark sur les ressources partagées pour satisfaire ce gate.

Le générateur `tests/perf/seed-10k.mjs` exporte uniquement du SQL : il ne choisit pas une base et ne publie rien. Le seul orchestrateur autorisé est `qa-isolated.mjs`, qui cible sa base nouvellement créée. Les modules catalogue/sitemap sont appelés avec un client HTTPS privé, pas avec un mock de serveur. Le rapport daté reste dans `docs/evidence/qa-isolated-<nonce>.json`, y compris en cas d’échec et de nettoyage incomplet.

Les recettes A14-5 (Google réel), A20-3 (lecteur écran), A21-x (exploitation) et A22-x (production et signatures humaines) sont planifiées dans QA_PLAN ; aucune n’est attestée par cette notice. Les tests locaux de comptes n’envoient pas d’emails réels et les essais IA déterministes ne consomment pas le quota du produit.
