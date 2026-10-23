# Étape 3.4 — Revue d'architecture avant gel

23 octobre 2026. Base 2a1b2f6. Prérequis 3.3 approuvé par « Oui continue ».
Créations : docs/evidence/ARCHITECTURE_GATE.md, docs/adr/0007-prototype-outcome.md, docs/CHANGE_REQUESTS/CR-2026-001.md, ce rapport.
Modifications : docs/ARCHITECTURE.md, docs/CONTRACTS.md, docs/PROGRESS.md. Aucun code ni schéma antérieur corrigé sans autorisation de la CR.

Audit exécuté : harness Rust important kortex-abi et appelant Reply/Envelope::decode sur trois messages ; AJV strict appliqué aux mêmes objets et protocol.schema.json. Résultats Rust [true,true,true], schéma [false,false,false] ; deux commandes code 0, divergence constatée, pas un test de conformité réussi. Message data:null, error:null et capability.result payload vide. Reproduction détaillée et preuve dans le gate.

Revue 3.0–3.3 : livrables approuvés et limites/dérogations conservées. Tests locaux passés sur Chromium/WebKit/Windows ; Firefox et OS omis sur instruction. P3 validation ZIP limitée au banc Node. Ces preuves ne justifient pas le gel tant que l'incohérence protocolaire reste présente.

Décision technique proposée : NO-GO pour gel et phase 4. CR concrète préparée, périmètre, migration de binaire/hash, tests et rollback définis. Aucune signature humaine inventée. Statut awaiting_approval pour la décision de correction, pas pour approuver un gel malgré le défaut. Aucun FROZEN_V1 attribué, aucun code produit commencé.
Commit résultat : commit contenant ce rapport.

Contrôles du livrable de revue : node scripts/check-progress.mjs : seize étapes ordonnées, code 0 ; node tests/conformance/check.mjs : 23 cas historiques réussis, code 0 (ne couvre pas les trois nouveaux défauts) ; tests de secrets : six réussites ; git diff --cached --check : code 0.

Reprise CR-2026-001 autorisée par le propriétaire : correction du codec appliquée, neuf tests ABI et 46 cas croisés réussis ; nouveau guest identifié et rejoué dans Chromium/WebKit/Wasmtime avec 33 cas identiques et interruption CPU effective. Le NO-GO initial est remplacé par une recommandation GO proposée au périmètre local, à revoir humainement. Pas de FROZEN_V1. Rapport consolidé : docs/evidence/cr-2026-001.md.
