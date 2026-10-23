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
