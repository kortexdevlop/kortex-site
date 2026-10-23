# Phase 4 — Contrôle global

23 octobre 2026, Windows. Linux/macOS et Firefox omis sur instruction du propriétaire. Accord explicite pour enchaîner 4.5, 5.0, 5.1 et 5.2.

Matrice : cargo test --workspace --locked : 59 tests + un doctest réussis ; corpus spécification 26/26 ; corpus commun TypeScript/Rust 46 cas identiques, trois tests Node réussis. Le nouveau test enregistré rejoue init/render/ui.event/capability.result/action.invoke/shutdown. Cent actions supplémentaires passent ; après échauffement, différence d'allocations conservées mesurée par allocateur de test : 0 octet. File vidée à chaque action et registre vide après shutdown. Ce résultat porte sur ce scénario et ne constitue pas une preuve universelle d'absence de fuite.

Le moteur RecordedEngine et l'allocateur instrumenté sont uniquement dans tests/integration/core_conformance.rs, déclaré comme test Cargo, jamais exportés en production. Signature éphémère de fixture vérifiée via VerifiedTool ; aucune clé privée distribuée. Les événements enregistrés utilisent la même définition gelée que les handlers UiRenderer testés en 4.4.

Risques ouverts : drivers plateformes et chaîne .tool à venir ; focus réel à confirmer au raccordement Web ; disponibilité des APIs navigateur, stockage privé et permissions à traiter en 5.2. Aucun schéma/ABI gelé modifié ; aucune correction dispatcher nécessaire à ce contrôle. Critères locaux Phase 4 satisfaits dans le périmètre Windows accepté.
