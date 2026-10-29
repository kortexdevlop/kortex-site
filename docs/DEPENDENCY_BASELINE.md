# Baseline des dépendances — 23 octobre 2026

| Élément | Version exacte | Preuve / état |
|---|---|---|
| Rust / Cargo | 1.99.1 | compilateur stable et Cargo exécutés |
| wasm32-unknown-unknown | toolchain 1.99.1 | compilation puis exécution Node : answer() = 42 |
| Node système | 24.15.0 | exécuté ; pnpm utilise aussi son runtime embarqué 24.19.0 |
| pnpm | 11.25.0 | installation frozen réussie |
| Dioxus / dx | 0.7.10 | compilation et liaison Desktop Windows réussies ; fenêtre non ouverte |
| Wasmtime | 50.0.0 | compilation Windows et exécution du module Core WASM : 42 |
| Astro | 7.4.2 | build SSR Windows réussi avec adaptateur Cloudflare |
| @astrojs/cloudflare | 14.4.2 | build SSR réussi ; peer Astro ^7.4.0, Wrangler ^4.125.0 |
| Tailwind CSS | 4.3.3 | registre consulté ; non installé |
| TypeScript | 7.0.2 | CLI exécutée |
| Wrangler | 4.152.0 | CLI exécutée |
| Playwright | 1.64.1 | CLI exécutée ; navigateurs non testés |
| Vitest | 5.0.5 | CLI exécutée ; aucun test produit |
| Ajv / ajv-formats | 8.20.0 / 3.0.1 | installation verrouillée |
| axe-core | 4.14.0 | installation verrouillée |

Windows x64 : contrôles ci-dessus. Linux/macOS : non vérifiés. Chrome/Firefox/WebKit : non vérifiés. Aucune compatibilité globale déclarée avant les prototypes correspondants.

Sources : https://blog.rust-lang.org/ ; https://crates.io/crates/dioxus/0.7.10 ; https://crates.io/crates/wasmtime/50.0.0 ; https://docs.astro.build/en/guides/deploy/cloudflare/ ; métadonnées npm relevées avec pnpm view.

Politique : mises à jour explicites avec résolution et contrôles ; aucun latest en CI. Les versions applicatives relevées restent candidates tant que leur compilation n'est pas prouvée. Cargo.lock est vide car aucune crate active. Ne pas présenter ce lock comme une résolution de Wasmtime/Dioxus.

Reprise : cargo build --locked -j 2 et cargo run --locked -j 2 réussis dans le banc isolé .git/probe/rust, profil debug=0, incremental=false. Ajv et ajv-formats : acceptation/rejet du format email vérifiés ; axe-core chargé. Matrice multi-OS et interface graphique restent non vérifiées. Locks du banc isolé distincts du lock racine vide ; aucun verrouillage applicatif Rust revendiqué.

## Navigateurs — essais Windows du 23 octobre 2026

Playwright 1.64.1 : installation des navigateurs réussie. WebKit 27.2 (build 2370) : démarrage headless et instanciation du même probe.wasm, résultat 42, réussi. Firefox 157.0 (build 1555) : échec de lancement spawn UNKNOWN ; lancement direct --version confirme une erreur Windows de configuration côte-à-côte. Compatibilité Firefox non vérifiée.
GitHub : aucun remote local ; list_repositories renvoie une liste vide. Aucun runner Linux/macOS accessible pour ce projet. Ces cases restent non vérifiées.

## Dioxus Web/Desktop — preuve 3.2

Dioxus 0.7.10 et dx 0.7.10 : builds Web/Desktop et interactions confirmés sous Windows. Le prototype active explicitement web-sys 0.3.106 avec feature Location côté WASM : sans ce feature, dioxus-web history.rs échoue sur Window.location. Cette dépendance de feature est verrouillée dans le lockfile du prototype ; à reprendre dans le futur hôte Web. WebView2 observée : Edge 155.0.4283.62. Chrome for Testing 156.0.8078.4. Captures et essais : docs/evidence/spike-3.2.md. Aucune preuve Linux/macOS/Firefox supplémentaire.

## API locale — préparation 9.2, 27 octobre 2026

Ajouts verrouillés : Zod 4.6.5, @cloudflare/vitest-plugin 1.4.7 et @cloudflare/workers-types 5.20261027.1. Le plugin actuel remplace le nom historique vitest-pool-workers du plan ; sa peer dependency accepte Vitest 5.0.5 et il réutilise exactement Wrangler 4.152.0 / Miniflare 5.20261019.2-beta déjà présents. TypeScript 7.0.2, Ajv 8.20.0 et ajv-formats 3.0.1 conservés. Le lock est installé puis contrôlé frozen. pnpm a inscrit l'exception ciblée d'âge du paquet de types publié ce jour ; aucune exception globale ni mise à jour des moteurs existants.

La date de compatibilité API est 2026-10-26 : le runtime verrouillé refuse strictement 2026-10-27 dans Vitest. Ce premier échec de démarrage a été corrigé par une date supportée commune au build/dev/tests, sans mise à jour silencieuse de workerd. Les types actuels utilisent Cloudflare.Env / GlobalProps, vérifiés au typecheck. Tests : trois groupes DTO Node et six tests dans workerd, bindings D1/R2 réels locaux ; build dry-run et HTTP dev local 200/404. Sources : [intégration Vitest officielle](https://developers.cloudflare.com/workers/testing/vitest-integration/) et métadonnées npm des versions précises. Aucune preuve distante ni plateforme exclue ajoutée.
# Prévisualisation design — 10.0–10.2, 27 octobre 2026

Astro 7.4.2 et Tailwind CSS / @tailwindcss/vite 4.3.3 installés et verrouillés dans le workspace website. Build statique Windows et vérification des composants Astro réussis ; atelier injecté en développement uniquement. @astrojs/check 0.9.12 déclare une peer TypeScript ^5 ou ^6 : TypeScript **6.0.3 est donc limité au workspace website**. TypeScript 7.0.2 reste inchangé pour les autres workspaces ; aucune exception globale aux peers ni à l’âge de publication. Le lock associe désormais Vite aux peers optionnelles jiti/yaml apportées par Astro ; les 3 tests DTO et 26 tests Worker ont été rejoués avec succès. Installation frozen contrôlée. Tests design sous Chromium/WebKit Windows ; Linux/macOS et Firefox exclus sur instruction du propriétaire.

## Bibliothèque et SSR — lot 10.3 / 10.4 / 11.0

@astrojs/cloudflare 14.4.2, @astrojs/preact 6.1.1 et Preact 11.0.1, versions exactes. Preact 11.1.0 publié ce jour a été écarté : l’exception d’âge automatiquement ajoutée par pnpm a été retirée, 11.0.1 antérieur conservé sans nouvelle exception. TypeScript 6.0.3 / @astrojs/check 0.9.12 inchangés ; @types/node 24.10.0 pour les tests Node et la frontière serveur ; déclarations Worker globales non activées dans le tsconfig mixte DOM. Ajv 8.20.0 / ajv-formats 3.0.1 et esbuild 0.28.2 déjà baselinés génèrent des validateurs ESM autonomes compatibles workerd. Wrangler 4.152.0 et date 2026-10-26 restent inchangés. Aucune mise à jour silencieuse des moteurs ni contrat gelé.

## Distribution Windows et canaux — 29 octobre 2026

Dioxus CLI 0.7.10 (archive Windows déjà épinglée SHA-256), NSIS 3.11 réellement exécuté, installateur EXE produit par dx bundle et WebView2 hors ligne silencieux. Le chemin µ exige la reprise MakeNSIS UTF-8 documentée dans DESKTOP_DISTRIBUTION.md. TypeScript 6.0.3 via alias typescript-ast reste limité à l’analyse AST, TypeScript CLI 7.0.2 inchangé. Aucune chaîne de signature Authenticode fournie : staging non signé explicitement, stable bloqué. Linux/macOS/Firefox exclus. Les hypothèses historiques de lock vide/GitHub absent en début de document ne décrivent plus le workspace courant : Cargo.lock et pnpm-lock.yaml sont résolus, le dépôt public kortexdevlop/kortex-site est actif.
