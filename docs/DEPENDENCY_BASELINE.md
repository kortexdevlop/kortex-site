# Baseline des dépendances — 23 octobre 2026

| Élément | Version exacte | Preuve / état |
|---|---|---|
| Rust / Cargo | 1.99.1 | compilateur stable et Cargo exécutés |
| wasm32-unknown-unknown | toolchain 1.99.1 | compilation puis exécution Node : answer() = 42 |
| Node système | 24.15.0 | exécuté ; pnpm utilise aussi son runtime embarqué 24.19.0 |
| pnpm | 11.25.0 | installation frozen réussie |
| Dioxus / dx | 0.7.10 | crate stable disponible, dx exécuté ; hôtes non compilés |
| Wasmtime | 50.0.0 | cargo info réussi ; compilation/exécution non vérifiées |
| Astro | 7.4.2 | registre npm consulté ; build non vérifié |
| @astrojs/cloudflare | 14.4.2 | peer Astro ^7.4.0, Wrangler ^4.125.0 ; build non vérifié |
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
