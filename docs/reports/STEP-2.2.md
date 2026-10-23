# Étape 2.2 — Crate ABI portable

23 octobre 2026. Windows, Rust/Cargo 1.99.1. Base 44b0780.
Prérequis 2.1 approuvé par « Oui continue ».
Créations : crates/kortex-abi/Cargo.toml, src/lib.rs, src/wire.rs, src/pointer.rs, tests/wire.rs, ce rapport.
Modifications : Cargo.toml (premier membre actif), Cargo.lock (résolution réelle), CONTRACTS.md, PROGRESS.md.
Dépendances exactes vérifiées via cargo search puis résolution : serde 1.0.229, serde_json 1.0.154 ; pas de thiserror nécessaire. Aucun moteur d'exécution.
Types : sept RequestKind, Envelope, Reply, HostError. Constantes ABI et plafonds B.2 ; pack/unpack et contrôle de plage u32, statut enqueue 0/négatif/positif inconnu.
Tests : roundtrip requête/réponses ; version/kind/UUID/payload invalides ; JSON vide, UTF-8 invalide et 1 MiB+1 ; pack/unpack bit de signe ; pointeur débordant, mémoire excessive, limite exacte ; code enqueue inconnu.
Limites : aucune allocation/exécution guest ni gestion d'instances ; ces comportements relèvent des étapes suivantes. Linux/macOS omis sur instruction antérieure, pas de compatibilité runtime déclarée.
Statut awaiting_approval, aucun gel. Commit résultat : commit contenant ce rapport.

Résultats : cargo test -p kortex-abi --locked : sept tests réussis, code 0. cargo check -p kortex-abi --target wasm32-unknown-unknown --locked : code 0. cargo fmt --all : code 0. node scripts/validate-schema.mjs : 26 contrôles réussis, code 0. node scripts/check-progress.mjs : huit étapes ordonnées, code 0.
