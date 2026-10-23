# Étape 4.1 — Cœur Runtime

23 octobre 2026. Base bb0a4f7. Prérequis 4.0 approuvé par « Oui continue ».
Créations : crates/kortex-core/Cargo.toml, src/lib.rs, src/lifecycle.rs, src/dispatcher.rs, src/errors.rs, tests/lifecycle.rs ; docs/evidence/runtime-4.1.md et ce rapport.
Modifications : Cargo.toml, Cargo.lock, PROGRESS.md. Aucun contrat gelé modifié.

Registre d'instances, lifecycle explicite, FIFO bornée/corrélation, entrées/sorties d'action validées, erreurs et nettoyage, délais via horloge injectée. GuestEngine asynchrone agnostique plateforme. VerifiedTool impose vérification de signature réelle à sa construction ; extraction et cohérence des fichiers signés restent une obligation de l'adaptateur qui sera implémenté aux étapes prévues.

Résultats locaux : dix tests Runtime réussis ; compilation native et WASM réussie. Délais testés par horloge simulée et abandon de future, pas par exécution moteur pendant 30 secondes. Une erreur de propagation d'échec cleanup a été trouvée par le test et corrigée. Détails et limites dans la preuve.

Critères locaux satisfaits : transitions illégales refusées, file 128, timeout, crash suivi de nettoyage, aucune instance conservée après shutdown. La libération effective des ressources externes dépend du contrat RAII/unload de l'adaptateur réel. Les stubs unitaires n'exposent aucun service produit.
Statut awaiting_approval. Commit résultat : commit contenant ce rapport.

Régressions finales : cargo test --workspace --locked : 28 tests et un doctest réussis, code 0 ; pnpm --filter @kortex/contracts test : trois tests et 46 cas identiques, code 0 ; node scripts/check-progress.mjs : 18 étapes ordonnées, code 0 ; node scripts/validate-schema.mjs : 41 contrôles, code 0 ; node tests/conformance/check.mjs : 26 cas, code 0.
