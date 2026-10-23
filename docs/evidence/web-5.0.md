# Preuve 5.0

Windows, 23 octobre 2026. Build dx 0.7.10 réussi, sortie locale : tests/spikes/dioxus-render/.git/probe/rust/target/dx/kortex-runtime-web/debug/web/public. Copie dans target/web-dist pour le serveur de tests. Cache réutilisé du prototype ; aucune modification de celui-ci.

Depuis runtime/web : CARGO_TARGET_DIR absolu vers ce cache, CARGO_PROFILE_DEV_DEBUG=0 et CARGO_INCREMENTAL=0 ; dx build --platform web --debug-symbols false. Depuis racine : cargo check -p kortex-runtime-web --locked ; pnpm exec playwright test -c tests/web/playwright.config.ts tests/web/shell.spec.ts : quatre réussites. Le hook Dioxus utilise uniquement du code hôte statique et le canal de données send/recv ; aucune interpolation de code guest.
