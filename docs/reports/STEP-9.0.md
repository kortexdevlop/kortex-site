# Étape 9.0 — Catalogue D1 indexé

Date : 2026-10-27. Base du lot : 02b0f6d. Résultat : 8ffb37d. Prérequis : gate 8.5 du même lot passé. Autorisation : propriétaire, « Enchaîne les 4 prochaines étapes ». Dernière étape du lot ; 9.1 non démarrée.

0001_catalog.sql crée catégories, outils, versions, actions et événements. Les clés composites empêchent les versions/actions dupliquées ; le pointeur courant référence une version publiée du même outil. ID et contenus de versions sont immuables, journal append-only, actions verrouillées après publication et transitions de versions sans retour en draft. Les quatre index partiels couvrent les deux tris stables et le filtre catégorie. Aucun binaire en base. DATABASE.md décrit colonnes, diagramme, transactions, migration et limites ; infra/environments.md prévoit KORTEX_DB sans inventer d'ID cloud.

| Commande | Date / environnement | Sortie | Résultat |
|---|---|---:|---|
| node --test database/tests/catalog.test.mjs | 2026-10-27, Node 24.15, SQLite 3.51.3 réel | 0 | 10 000 outils/actions, FK clean, 13 refus, pagination stable, archivage/révocation transactionnels |
| node database/tests/d1-local.mjs | Windows, Wrangler 4.152.0, D1 local réel | 0 | Migration appliquée puis aucun pending au rejeu ; 10 000 outils/actions ; 5 refus ; 4 plans indexés |

Les deux moteurs utilisent les quatre index idx_tools_recent/name/category_recent/category_name avec SEARCH USING COVERING INDEX, sans scan global ni tri temporaire pour les requêtes testées. Les 10 000 métadonnées sont synthétiques ; aucun guest n'est compilé/exécuté en cloud.

Écart explicite : sqlite3 CLI absent, prévalidation par le vrai SQLite intégré à Node et essai supplémentaire du moteur D1 local. D1 local interdit sqlite_version() ; cette interrogation de diagnostic a été retirée, sans attribuer une version interne D1 imaginaire. Les scripts Wrangler exigent --local et refusent --remote. Aucune création de ressource distante. Le rejeu est celui de l'outil de migration, pas une réexécution brute de CREATE TABLE.

Le schéma SQL protège les invariants de stockage, pas toute la sémantique des schémas ni les signatures. Le futur service devra appliquer les validateurs gelés et la publication atomique R2/D1. Aucune API ni validation de charge distante revendiquée. Logs/hash/résultats : docs/evidence/validation-8.3-9.0.json. Critère local satisfait.
