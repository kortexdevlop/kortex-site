# Étape 4.0 — SDK guest réutilisable

23 octobre 2026. Base 8a57069. Le propriétaire a répondu « Oui continue » à la demande explicite de revue 3.4 corrigée et gel avant SDK. PROGRESS consigne le gel approuvé et CONTRACTS les empreintes des cinq schémas existants, sans attribution d'approbation à un tiers.
Créations : crates/kortex-sdk/Cargo.toml, src/lib.rs, src/ffi.rs, src/capability.rs, tests/dispatch.rs, docs/evidence/sdk-4.0.md, ce rapport.
Modifications : Cargo.toml, Cargo.lock ; CONTRACTS.md (gel reçu et ownership), PROGRESS.md, ARCHITECTURE_GATE.md (approbation reçue). Aucune modification des schémas ou signatures ABI gelés.

Tool::handle(Request)->Reply, Dispatcher et macro export_tool!, buffers enregistrés avec ownership exact, file Host::enqueue et callbacks corrélés sur capability_request_id. request_id de réponse conservé. La capacité reste autorisée par l'hôte, pas par le SDK. Aucun WASI/OS importé.

Résultats observés : tests SDK sept réussites et un doctest, cible WASM compilable, fuzz déterministe de 20000 couples invalides ; erreurs, commandes multiples et résultats hors ordre testés. Guest macro réellement compilé/instancié sous Node : 10000 dispatchs et corrélation asynchrone simulée dans le harness réussis. Preuve et reproduction dans sdk-4.0.md.

Limites : transport acceptant/refusant dans tests uniquement, aucun effet réel de capacité. Les callbacks sont délivrés dans un dispatch futur, pas par réentrance synchrone. Panic natif Tool intercepté ; panic utilisateur abort sous WASM ou épuisement mémoire peut terminer l'instance en trap, sans unwind FFI ni garantie de Reply. L'hôte produit gérera cette interruption. Les budgets de module restent à imposer par le builder.
Statut awaiting_approval. Commit résultat : commit contenant ce rapport.

Régressions finales : cargo test --workspace --locked : 18 tests et un doctest réussis, code 0 ; pnpm --filter @kortex/contracts test : trois tests, 46 cas identiques, code 0 ; node scripts/validate-schema.mjs : 41 contrôles, code 0 ; node tests/conformance/check.mjs : 26 cas, code 0 ; node scripts/check-progress.mjs : 17 étapes ordonnées, code 0.
