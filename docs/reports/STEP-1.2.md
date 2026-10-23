# Étape 1.2 — Résultat local partiel

Base fe3d120, Windows, 23 octobre 2026. Étape 1.1 approuvée par « Continue ».
Versions et limites : docs/DEPENDENCY_BASELINE.md.

Pass : rustc/Cargo 1.99.1, installation target, compilation WASM puis exécution answer() = 42 ; pnpm install --frozen-lockfile code 0 ; CLI TypeScript/Vitest/Playwright/Wrangler exécutées. dx 0.7.10 exécuté.
Échecs corrigés : globs Cargo vers dossiers absents remplacés par members vide ; génération Cargo.lock réussie. Scripts esbuild/workerd autorisés explicitement dans pnpm-workspace.yaml, installation réussie après refus initial.
Écart au plan : le workspace vide exige members=[] pour être valide ; le fichier pnpm-workspace.yaml a nécessité la configuration des scripts de dépendances.

Créations : rust-toolchain.toml, Cargo.lock, docs/DEPENDENCY_BASELINE.md, ce rapport.
Modifications : Cargo.toml, package.json, pnpm-lock.yaml, pnpm-workspace.yaml, docs/PROGRESS.md.

Non vérifié : compilation Wasmtime et hôtes Dioxus, build Astro Cloudflare, Linux/macOS, navigateurs. Les versions de ces composants sont candidates, pas une matrice validée. L'étape 1.2 reste active ; critères d'acceptation complets non satisfaits. Aucun gel ni approbation demandés sur une baseline incomplète.
Commit : commit contenant ce rapport, voir git log.

## Reprise du 23 octobre 2026

Build SSR Astro 7.4.2 + @astrojs/cloudflare 14.4.2 : code 0, Server built in 5.04s. Adaptateur active par défaut bindings IMAGES et SESSION : à configurer explicitement lors de la création du site. Aucun déploiement.
Nouvelle installation racine pnpm --frozen-lockfile : code 0.
Compilation combinée Wasmtime 50.0.0 et Dioxus Desktop 0.7.10 : échec code 101, espace disque insuffisant (Windows erreur 112). Cette preuve ne permet pas de conclure sur la compatibilité. Fichiers de compilation dans .git/probe/rust/target.
Nettoyage ciblé de ce dossier tenté ; action refusée par la politique automatique d'exécution. Aucun fichier utilisateur supprimé. Étape toujours active, sans validation.

## Reprise après libération d'espace — 23 octobre 2026

Espace initial constaté : 10 114 109 440 octets libres. Banc isolé déjà configuré debug=0 et incremental=false ; compilation limitée à deux tâches.
- cargo build --locked -j 2 (.git/probe/rust) : code 0, 3 min 39 s. Dioxus Desktop 0.7.10 et Wasmtime 50.0.0 compilés et liés sous Windows.
- cargo run --locked -j 2 : code 0. Wasmtime executed WASM = 42; Dioxus Desktop linked. Aucune fenêtre Dioxus ouverte : rendu et interaction non vérifiés.
- cargo metadata --locked --no-deps --format-version 1 (racine) : code 0, workspace vide valide.
- pnpm install --frozen-lockfile : code 0.
- Ajv + ajv-formats : acceptation d'une adresse valide et rejet d'une chaîne invalide, code 0 ; axe-core 4.14.0 chargé.

Les échecs de disque précédents sont résolus. La baseline Windows locale possède désormais ses preuves de compilation principales. Linux/macOS et navigateurs restent non vérifiés ; étape active, aucune validation globale déclarée. Le banc isolé est conservé localement, sans ajout d'application au monorepo.
