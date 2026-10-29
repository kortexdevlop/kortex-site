# Plans de maintenance staging

Déposer ici un plan JSON relu avant déclenchement manuel du workflow `tools.yml`. Le plan fixe `schema_version:1`, `channel:staging`, une action, un `request_id` UUID stable et une cible explicite. Rejouer exactement le même plan après une coupure. Un même identifiant ne doit jamais désigner deux requêtes.

Rollback outil : `action:rollback`, `target:tool`, `tool_id`, `version` saine visée et `expected_current_version` observée. Rollback hôte : `target:api|website|runtime`, `version_id` ancien et `expected_version_id` actuel. Runtime exige aussi `channels`, enveloppe signée avec nouveau compteur, version et bundle compatibles. Desktop ne fait qu’annoncer le canal signé ; aucune installation forcée. Un plan Desktop n’est pas une preuve d’installation.

Retrait : `action:revoke`, `tool_id`, `version`, `sha256`, `reason` et `revocations`, document public déjà signé hors CI, compteur supérieur et tombstones conservés. La racine privée reste hors GitHub. Le document signé est installé avant la transaction de retrait ; en cas de coupure, reprendre le même plan.

Pas de plan exemple directement exécutable ni de cible production : celle-ci demeure refusée. Voir `docs/reports/RELEASE_PIPELINE_GATE.md`.
