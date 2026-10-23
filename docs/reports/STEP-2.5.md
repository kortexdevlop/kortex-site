# Étape 2.5 — Suite de conformité

23 octobre 2026, Windows/Node 24.15.0/Rust 1.99.1. Base b046acc.
Prérequis 2.4 approuvé par « Oui continue ».
Créations : tests/conformance/README.md, v1.cases.json, check.mjs ; docs/adr/0006-versioning.md ; ce rapport.
Modifications : CONTRACTS.md et PROGRESS.md.
Corpus indépendant des moteurs : négociation, init/render/action.invoke, erreurs structurées, ordre, corrélation, versions et permissions. Refus UNSUPPORTED_ABI avant toute instanciation dans la spécification ; runner n'instancie rien.
La vérification de traces n'est pas un runtime exécutant. Les traces sont des fixtures explicites ; aucune preuve cross-host ou signature cryptographique présentée comme acquise. Prototypes P1–P5 requis avant gel. Linux/macOS omis sur instruction antérieure.
Statut awaiting_approval, aucun gel. Commit résultat : commit contenant ce rapport.

Résultats : node tests/conformance/check.mjs : 23 cas réussis, code 0 ; log local .git/probe/conformance-2.5.log, SHA-256 16FD44CD00A397DE139D25852416BF898F058D906EADA87D6AC5616FD8FAA6CD. cargo test -p kortex-abi --locked : sept tests réussis, code 0. pnpm --filter @kortex/contracts test : trois tests réussis, 33 cas croisés identiques, code 0. node scripts/check-progress.mjs : onze étapes ordonnées, code 0.
