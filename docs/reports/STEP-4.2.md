# Étape 4.2 — Permissions et adaptateur de capacités

23 octobre 2026, base 2696d7f. Prérequis 4.1 approuvé par « Oui continue ».
Créations : crates/kortex-capabilities/Cargo.toml, src/lib.rs, src/policy.rs, src/handles.rs, src/audit.rs, tests/deny_by_default.rs ; docs/evidence/capabilities-4.2.md et ce rapport.
Modifications : Cargo.toml/Cargo.lock, crates/kortex-core/Cargo.toml, src/dispatcher.rs, src/lib.rs, tests/lifecycle.rs, docs/PROGRESS.md. Les membres/dépendances et hooks Runtime supplémentaires sont nécessaires pour raccorder effectivement la policy à l'identité chargée et tester le critère des appels sensibles ; aucun ancien contrat gelé modifié.

Livrable : deny/ask/allow-once/allow-session, grants par outil/version/instance/ressource, gestes séparés, handles opaques expirables, journal minimisé et CapabilityHost abstrait. Tous les sept chemins sensibles de Gateway sont testés après politique ; le bridge Runtime prend l'identité du manifeste et refuse les permissions non déclarées. Ressources réseau canoniques, contrôle de redirections et DNS, budgets de 10 MiB fichiers et enveloppes JSON inchangées.

Résultats : 16 tests de capacités, 11 core, 45 workspace et un doctest réussis ; compile WASM réussie ; régressions schémas/conformité/contrats croisés réussies. Preuves détaillées et limites d'adaptateur dans capabilities-4.2.md.

Critères locaux satisfaits : refus par défaut, grant A non valide pour B/version/instance, redirection non autorisée arrêtée avant effet cible. Limites : drivers système réels et UI de consentement à implémenter dans les hôtes ; DNS pinning/lecture bornée/échéance doivent être respectés par leurs drivers, sans quoi la capacité reste Unsupported. Nettoyage de session hôte via shutdown_with_capabilities ou révocation explicite après erreur et destruction du Gateway au remplacement du Runtime. Aucun mock permanent, seulement stubs de tests.
Statut awaiting_approval. Commit résultat : commit contenant ce rapport.

Contrôles finaux : node scripts/check-progress.mjs : 19 étapes ordonnées, code 0 ; cinq empreintes de schémas gelés vérifiées sans changement. Les sous-dépendances JSON Schema transitoirement résolues en 0.60.1 ont été remises aux versions de lock 0.60.0 avant régression complète.
