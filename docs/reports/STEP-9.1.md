# Étape 9.1 — État de publication et intégrité R2

Date : 2026-10-27. Base : 751a84f. Résultat : 8237f5e. Prérequis : 9.0 validée. Autorisation : propriétaire, terminer 9.1 puis préparer 9.2 et s'arrêter dès qu'une connexion Cloudflare est nécessaire. Linux/macOS et Firefox restent exclus.

0002_publication.sql ajoute artifacts, signing_keys, publication_jobs, publication_job_events et revocations. Les artefacts lient une identité/version à une clé R2 et un SHA immuables. Les promotions exigent des attestations HEAD/relecture/signature cohérentes et une clé active ; les états ne reculent pas. Une répétition d'UUID identique n'ajoute ni job ni événement ; une charge différente est refusée. Les révocations sont append-only, interdisent la sélection d'une version révoquée et exigent de retirer/déplacer d'abord les pointeurs actifs. Un rollback légitime peut sélectionner une ancienne version publiée sans modifier ses octets.

| Commande | Date / environnement | Sortie | Résultat |
|---|---|---:|---|
| node --test database/tests/publication.test.mjs | 2026-10-27, Windows, SQLite 3.51.3 / Node 24.15 | 0 | 30 refus, idempotence, publication contrôlée, rollback transactionnel, révocations, zéro erreur FK |
| node database/tests/publication-d1-local.mjs | Windows, Wrangler 4.152.0 / D1 local | 0 | 2 migrations appliquées puis aucun pending, 12 refus, 1 job à répétition, 4 événements jusqu'à revoked, FK clean |
| node --test database/tests/catalog.test.mjs | Windows, SQLite réel | 0 | Régression 10 000 outils/index inchangée |
| node database/tests/d1-local.mjs | Windows, D1 local | 0 | Régression catalogue 10 000 outils et 4 index ; banc 0001 isolé des migrations suivantes |

Le banc 9.0 copie désormais 0001 dans son propre répertoire de migrations : son jeu historique publié sans jobs ne prétend pas satisfaire les nouvelles attestations de 0002. 0001 reste inchangée. Le journal de transitions utilise une clé entière et une FK vers le vrai UUID de requête, sans UUID synthétiques dérivés susceptibles de collision.

infra/r2-policy.md précise buckets privés prévus, PUT conditionnel, SHA indépendant de l'ETag, rétention, cache, révocations et absence d'écrasement. DATABASE.md décrit transactions et invariants. Les schémas publics gelés ne changent pas.

Limite explicite : le SQL ne fait ni HEAD/GET R2 ni Ed25519. Les attestations des fixtures sont synthétiques pour tester la machine d'état. Le futur service devra effectuer ces opérations réelles avant d'écrire les attestations. NOT_FOUND/HASH_CONFLICT/IDEMPOTENCY_CONFLICT ont les correspondances contractuelles 404/409 ; les tests portent sur les erreurs SQL, pas sur des endpoints HTTP de publication qui n'existent pas encore. Aucun compte, bucket, upload ou déploiement distant. Preuves : docs/evidence/validation-9.1.json. Critères locaux de schéma satisfaits ; poursuite 9.2 autorisée dans la demande utilisateur.

Contrôle final du même lot : les INSERT sont également protégés contre INSERT OR REPLACE, dont les suppressions peuvent éviter les triggers DELETE par défaut. Sept cas SQLite et trois cas D1 supplémentaires refusent le remplacement des identités, clés et journaux. Cette migration reste exclusivement locale, sans déploiement à migrer.
