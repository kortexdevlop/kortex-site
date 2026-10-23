# Étape 2.1 — Manifestes et actions

23 octobre 2026. Windows, Node 24.15.0. Base 0624d4f.
Prérequis 2.0 approuvé par « Oui,continue ».
Créations : deux JSON Schemas 2020-12, quatre fixtures canoniques/invalides, scripts/validate-schema.mjs, ce rapport.
Modifications : CONTRACTS.md (KTX-SCHEMA-001 : bornes et normalisation candidates), PROGRESS.md.
AJV compile les validateurs JS depuis les schémas ; runner temporaire consommable par TS. Pas de fichier TS généré à cette étape, package partagé prévu en 2.4.
Compléments sémantiques : unicité des identifiants d'action, profondeur et bornes cohérentes. Ces contraintes ne sont pas garanties par le schéma seul.
Bornes textuelles et profondeur proposées explicitement, à revoir avant gel ; aucune valeur implicite présentée comme issue du plan.
Critères locaux : fixtures attendues acceptées/rejetées, indépendance ordre des clés vérifiée. Validation Rust croisée ultérieure, non réalisée ici.
Statut awaiting_approval. Aucun gel. Commit résultat : commit contenant ce rapport.

Résultats : node scripts/validate-schema.mjs : 26 contrôles réussis, code 0 ; log local .git/probe/schema-2.1.log, SHA-256 03F9511D838BBA7C8D011E15BAF8DC8300EBAD98A0858FFCD660706DC98E2C62. node scripts/check-progress.mjs : code 0, sept étapes ordonnées. node --test tests/config/secrets.test.mjs : six réussites, code 0. git diff --cached --check : code 0.
