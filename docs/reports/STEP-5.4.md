# Étape 5.4 — Hôte Web isolé

Finalisation : 2026-10-27. Base : ceac36a. Résultat : commit contenant ce rapport.
Lot 5.4, 6.0 et 6.1 explicitement demandé par le propriétaire (« Enchaîne les trois prochaines étapes », puis « Termine »). Aucun contrat gelé modifié.

Le vrai shell Dioxus et un ZIP signé s’exécutent dans une iframe sur une seconde origine localhost. HELLO vérifie origine, fenêtre source, version et nonce ; READY crée une session ; OPEN exige session et nonce. Les réponses ciblent l’origine exacte. Le parent prototype ouvre l’outil et affiche RESULT. Aucun objet moteur n’est transmis au parent.

La liaison Rust/JavaScript utilise wasm-bindgen et l’entrée canonique `/adapter/interop.js`. Les validateurs JSON sont générés au build et les schémas d’actions interprétés sans compilation JavaScript dynamique. La CSP n’exige ni unsafe-eval JavaScript ni script inline. Les contrôles Rust des origines sont raccordés. La configuration de confiance provient du déploiement, jamais des messages du parent.

| Commande | Environnement | Résultat |
|---|---|---|
| `pnpm --filter @kortex/web-adapter typecheck` | Windows, 2026-10-23 | Sortie 0 |
| `pnpm --filter @kortex/web-adapter test` | Chromium/WebKit, 2026-10-23 | 26 tests réussis |
| `dx build --platform web --debug-symbols false` | runtime/web, Windows, 2026-10-23 | Sortie 0, vrai shell compilé |
| `pnpm exec playwright test tests/web` | Chromium/WebKit | 8 tests : shell, focus et iframe |
| `node --test tests/contracts/cross-language.test.mjs` | Windows, 2026-10-23 | 3 tests, matrice commune de 46 cas |
| `node tests/contracts/browser-csp.mjs` | Windows, 2026-10-23 | 33 cas manifest/contrat identiques |

Les scénarios iframe vérifient le rejet d’une mauvaise version et du rejeu de HELLO, l’ouverture signée réelle, le refus de lecture du stockage et de l’objet hôte par le parent, le blocage d’un script inline, le refus d’un parent non autorisé par frame-ancestors, puis le crash et le redémarrage du Worker. Le bouton du prototype parent mène à output=ready.

Échecs corrigés : deux compilations JavaScript dynamiques interdites par CSP ; encodage UTF-8 du prototype parent ; fermeture des connexions de test. Linux/macOS et Firefox sont exclus à la demande du propriétaire. Déploiement prouvé sur deux ports localhost distincts ; aucun hébergement public revendiqué.

Preuves : `tests/web/embed.spec.ts`, `docs/evidence/web-host.md`, `docs/evidence/validation-5.4-6.1.json`. Autorisation du lot consignée dans `docs/PROGRESS.md`.
