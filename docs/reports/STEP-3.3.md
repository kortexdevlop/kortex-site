# Étape 3.3 — Isolation et archives

23 octobre 2026, base e94e0c3. Prérequis 3.2 approuvé par « Oui continue ».
Créations : tests/spikes/sandbox-host/index.html, iframe.html, archive.test.ts ; docs/evidence/spike-3.3.md ; ce rapport.
Modifications : CONTRACTS.md et PROGRESS.md. Aucune dépendance supplémentaire : crypto/zlib natifs Node et Playwright existant.
Résultats : trois tests Playwright réussis, code 0. Parent/runtime sur ports distincts, refus DOM parent, contrôles origine/source/nonce, CSP, Worker WASM réel, checksum de téléchargement. Banc ZIP Node vérifie signature détachée et rejets obligatoires, stored/deflate acceptés. Budget et mesures observées dans la preuve.
Limites explicites : vérification ZIP côté Node dans ce prototype ; pas de parseur produit navigateur livré, pas de pic mémoire complet du navigateur, aucun audit ZIP exhaustif ni clé produit. Le faux parent et les métadonnées hostiles sont des fixtures de test, aucun service simulé présenté comme production. Firefox/Linux/macOS omis selon décisions antérieures.
Critères locaux du prototype satisfaits ; politiques ZIP proposées avant gel, plafonds B.2 conservés. Statut awaiting_approval. Commit résultat : commit contenant ce rapport.

Régressions : node scripts/check-progress.mjs : quinze étapes ordonnées, code 0 ; node tests/conformance/check.mjs : 23 cas réussis, code 0 ; node --test tests/config/secrets.test.mjs : six réussites, code 0 ; git diff --cached --check : code 0.
