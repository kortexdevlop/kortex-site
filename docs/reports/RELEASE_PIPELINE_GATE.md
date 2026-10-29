# Gate de reprise release — staging

Les commandes `promote-release.mjs`, `rollback-release.mjs` et `revoke-artifact.mjs` consomment un plan explicite et stable. Le publisher GitHub des outils réutilise le même moteur de promotion : sauvegarde avant écriture, objet adressé par hash, transaction admin D1, vérification de visibilité, reprise par `request_id`. Un replay déjà engagé est réconcilié à partir du journal D1 et ne réapplique pas un ancien pointeur.

## Matrice pannes / récupération

| Coupure ou risque | Comportement attendu | Vérification |
|---|---|---|
| Sauvegarde indisponible | Aucune mutation | Test Node `backup failure` |
| Archive R2 envoyée, D1 non publié | Aucune fiche visible ; reprendre même plan | Drill réel privé `before_publish_invisible` |
| Commit D1 réussi, réponse perdue | Retour de l’opération originale ; une seule publication | Drill `after_publish_reconciled`, compteur admin |
| Ancien publish rejoué après rollback | Ne repointe pas sur la version remplacée | Drill `replay_does_not_repoint` |
| Rollback concurrent | Refus si version courante différente de `expected_current_version` | API tests CAS et drill réel rollback |
| Retrait urgent | Document public signé installé avant retrait transactionnel ; téléchargement 410 | Drill signé, aucun secret racine en CI |
| Historique réécrit | Triggers D1 append-only ; chaîne locale vérifiée au redémarrage | DELETE distant refusé et test journal altéré |
| Mauvais canal ou production | Refus avant mutation | Tests Node ; production non provisionnée |
| Runtime incompatible/signature invalide | Vérification du canal avant changement Worker ; bundle vérifié ensuite | Validateur partagé + garde publisher ; rollback Runtime historique 19.4 |
| Migration non réversible | Aucun restore implicite ni exécution de down migration | Retour de version Worker conserve schéma ; réparation compensatoire à définir pour chaque migration |

Le drill courant crée ses propres D1, R2, API et clés de test ; il ne change aucun outil réel. Les fixtures de release contiennent de vrais octets WASM et ZIP canonique, vérifiés par le véritable service admin. Les doubles de test Node servent aux cas limites locaux ; ils ne sont pas la preuve du drill distant.

## Sauvegardes et limites de rétention

Avant chaque nouvelle opération, un bookmark D1 Time Travel et les métadonnées concernées sont enregistrés dans `release-backups/<request_id>.json`. Rejouer une opération vérifie le fingerprint du backup existant. Le journal local est chaîné par SHA-256 ; les événements sont également déposés sous des clés uniques `release-audit`. GitHub publie les étapes dans son résumé et conserve les journaux comme artefacts du job. Aucune notification externe n’est envoyée.

Les archives R2 utilisent une clé `tools/id/version/sha.tool`, une écriture conditionnelle et une relecture du hash. Ce stockage adressé par contenu et les triggers D1 apportent l’immutabilité applicative. **Ce n’est pas une garantie WORM contre un administrateur du compte.** R2 ne fournit pas ici un historique automatique activé d’anciennes versions d’une même clé ; aucune politique globale de bucket lock n’est imposée sans durée de rétention validée. Les backups sont sous des clés nouvelles, jamais sous un nom « latest » écrasable.

[D1 Time Travel](https://developers.cloudflare.com/d1/reference/time-travel/) conserve sept jours en Free et trente en Paid, selon le plan effectif. Le drill vérifie un bookmark réel via Wrangler. Il n’effectue aucune restauration destructive. Une restauration D1 de production nécessite un plan incident dédié, une décision humaine et une vérification des données arrivées depuis le bookmark. Les [locks R2](https://developers.cloudflare.com/r2/buckets/bucket-locks/) restent une décision d’exploitation 21.x avec politique de rétention ; ne pas verrouiller les buckets temporaires.

## Modes opératoires

En CI, `tools.yml` propose `publish`, `rollback` ou `revoke`, avec plan relu dans `infra/releases`. Branche main, identité GitHub OIDC exacte, environnement staging protégé et qualité du même SHA sont obligatoires. La politique admin staging doit autoriser explicitement les actions de maintenance ; les secrets racines privés restent absents de CI. Un retrait consomme un document de révocation public déjà signé hors CI.

Localement sur Windows, un opérateur autorisé utilise `node scripts/rollback-release.mjs --plan <fichier> --operator` avec bearer signé court fourni par l’environnement ; aucun bearer n’est demandé en argument. Les scripts de rollback API/site changent uniquement la version du Worker et conservent les bindings ; ils contrôlent que le schéma D1 n’a pas changé. Le rollback Desktop actualise le canal signé : il n’installe ni ne désinstalle un logiciel à distance.

Production demeure refusée, même avec des identifiants disponibles. La décision humaine, les gates Google/rétention/sécurité et le certificat Windows de production restent requis. Les essais Worker/Runtime et Desktop de 19.2–19.4 restent des preuves historiques distinctes ; le nouveau drill distant valide les transactions outils et la reprise, sans prétendre répéter une installation Desktop complète.
