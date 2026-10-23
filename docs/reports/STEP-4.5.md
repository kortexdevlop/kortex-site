# Étape 4.5 — Contrôle global du cœur

Base 56dd7c8. Autorisation du propriétaire : « Oui continue, Enchaînez les quatre prochaines étapes. »

Rapport et preuves : PHASE4_GATE.md ; tests/integration/core_conformance.rs ; tests/fixtures/events.ndjson. Nouveau test enregistré via crates/kortex-core/Cargo.toml pour que cargo test --workspace l'exécute réellement. Moteur faux réservé au test, pas de mock permanent. Aucun contrat modifié.

59 tests workspace + un doctest réussis ; 26 cas du corpus de conformité ; 46 cas communs Rust/TypeScript et trois tests Node réussis. Cent actions sans croissance des allocations conservées mesurées (delta 0), files vidées, nettoyage final. Windows uniquement selon dérogation du propriétaire. Étape validée dans le cadre de l'autorisation anticipée d'enchaînement ; poursuite vers 5.0.
