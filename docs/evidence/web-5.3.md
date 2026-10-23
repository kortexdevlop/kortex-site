# Preuve 5.3 — Admission archive signée

Windows, 23 octobre 2026. Tests loader.spec.ts dans Chromium et WebKit : signature valide puis tampering, clé inconnue, expiration, révocation et ABI incompatible ; ZIP signé avec manifest/contract/module réel ; résultat multiply 3*7=21 ; chemin ../escape, taille annoncée hostile, cache corrompu et MIME HTML refusés. Clés privées éphémères exclusivement dans le processus de test Node.

Commandes : pnpm --filter @kortex/web-adapter typecheck ; pnpm --filter @kortex/web-adapter test ; pnpm exec playwright test tests/web ; depuis runtime/web, dx build --platform web --debug-symbols false avec le cache CARGO_TARGET_DIR déjà documenté en 5.2. Distribution dx copiée dans target/web-dist avant le test final, incluant les scripts publics traités par dx. Aucun contournement de signature dans le serveur de test.

Limites : ZIP classique stored/deflate uniquement, pas de ZIP64/data descriptors ; test positif stored. Cache volontairement limité à un artefact courant par origine ; revalidation obligatoire à chaque lecture. Liste de clés publiques distribuée vide et marquée test-only : toute clé absente reste refusée. Production non déployée ; séparation iframe/CSP et handshake restent à 5.4.
