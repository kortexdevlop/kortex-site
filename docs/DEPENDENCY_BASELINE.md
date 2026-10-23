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
