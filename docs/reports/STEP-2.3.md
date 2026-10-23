# Étape 2.3 — UI, protocole et workflows

23 octobre 2026, Windows/Node 24.15.0/Rust 1.99.1. Base 29f5544.
Prérequis 2.2 approuvé par « Oui continue ».
Créations : ui.schema.json, protocol.schema.json, workflow.schema.json, ui.valid.json et workflow.valid.json, ce rapport.
Modifications : CONTRACTS.md, PROGRESS.md ; runner scripts/validate-schema.mjs étendu pour exécuter les tests obligatoires de cette étape.
Contrats candidats : props fermées des 18 nœuds, événements, labels accessibles, graphiques bornés ; DAG générique, action_ref, bindings JSON Pointer et confirmation ; capability.result corrélé. Tous les ajouts restent candidats au gel.
Rejets exercés : cycle, clés dupliquées, CSS URL, HTML brut, nœud inconnu, sortie d'action hors schéma. Schémas et invariants sémantiques distincts.
Fixtures croisées : UI et workflow sont destinés aux validateurs TS/Rust de 2.4. Aucun essai Rust sur ces fixtures déclaré à cette étape. Cargo tests ABI restent une régression du codec, sans preuve de renderer graphique.
Limites : accessibilité réelle, résolution des pointeurs dans les données et autorisations d'exécution relèvent des hôtes et du moteur de workflow.
Statut awaiting_approval. Commit résultat : commit contenant ce rapport.

Résultats finaux : node scripts/validate-schema.mjs : 41 contrôles réussis, code 0. cargo test -p kortex-abi --locked : sept tests réussis, code 0. node scripts/check-progress.mjs : neuf étapes ordonnées, code 0. Tests de secrets : six réussites. git diff --cached --check : code 0. Un premier échec de compilation AJV strictRequired du payload capability.result a été corrigé sans désactiver le mode strict.
