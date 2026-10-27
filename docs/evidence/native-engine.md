# Moteur natif — preuves 6.0 et 6.1

Wasmtime 50.0.0, Rust Windows MSVC ; Dioxus Desktop 0.7.10 avec WebView2 pour la présentation. Guest Core identique octet pour octet au guest Web de 3.0 : SHA-256 3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65.

Deux tests moteur : 100 appels multiply à 1200 ; boucle interrompue et buffer invalide refusé. Neuf tests ABI et 36 tests Core/capacités/validateurs partagés passent après les corrections de style Clippy. La commande Clippy complète demandée passe avec -D warnings.

La preuve Desktop pilote l’exécutable Windows et son DOM WebView2 par Playwright CDP. Le guest s’exécute dans l’acteur Wasmtime. Le test et le générateur sont conservés dans tests/native/desktop.cjs et tests/fixtures/make-tool.cjs. Capture : native-desktop.png. Instructions : runtime/desktop/README.md.

Résultats et empreintes des preuves : validation-5.4-6.1.json. Limites et échecs corrigés : rapports STEP-6.0 et STEP-6.1. Linux/macOS et Firefox n’ont pas été testés conformément à la demande.
