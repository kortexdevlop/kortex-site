# Contribuer et maintenir Kortex

Le dépôt est public, mais le propriétaire a choisi une [licence propriétaire](../LICENSE). La consultation/fork sur GitHub ne donne pas une licence open source générale. Avant d'intégrer une contribution externe, clarifier avec le titulaire les droits nécessaires ; aucune CLA signée ou cession n'est supposée.

## Proposer un changement

Ouvrir une branche courte, par exemple `codex/mon-outil`, et décrire problème, comportement obtenu, périmètre, tests et limites dans la PR. Ne pas ajouter de secret, données utilisateur, clé privée ou binaire de production non traçable. Le budget de services autorisé est 0 € : aucun achat ou changement automatique de forfait n'accompagne une PR.

Suivre [DEVELOPER_SETUP](DEVELOPER_SETUP.md) puis [TOOL_AUTHOR_GUIDE](TOOL_AUTHOR_GUIDE.md) pour un outil. Le sixième exemple Base64 est construit/testé sans branche conditionnelle dans les hôtes, preuve [21.1](reports/STEP-21.1.md). Une nouvelle permission ou surface de confiance demande une revue du modèle de menace et des dialogues de consentement.

## Revoir et valider

Chaque fichier versionné est rattaché à un rôle et une étape par [FILE_OWNERS](FILE_OWNERS.json). Il s'agit d'une responsabilité technique, pas d'une liste de personnes déjà nommées. Modifier la règle quand le périmètre de maintenance change. `node scripts/check-doc-links.mjs` refuse liens internes/ancres invalides et fichiers sans étape propriétaire ; `node scripts/check-progress.mjs` vérifie le registre ordonné.

Choisir les tests correspondant au changement, puis les gates globaux nécessaires avant livraison. Les quatre lanes CI Windows restent obligatoires ; ne pas marquer un échec/skipping critique comme réussi. Une PR ne reçoit pas les secrets de publication. `main`, environnement protégé, CI du même SHA et vérifications OIDC sont exigés pour publier. Les résultats locaux n'annulent pas les tests staging.

Les références/goldens figés ne sont pas régénérés pour faire passer le code. Examiner toute dérive de comportement ou de hash ; documenter la cause, conserver les observations en échec et appliquer la [politique de compatibilité](COMPATIBILITY_POLICY.md).

## Livrer et maintenir

Versions d'outils, Runtime, site/API et Desktop restent distinctes et traçables. Un correctif ne réécrit pas les octets d'une version publiée. Construire, signer, vérifier, publier par le chemin protégé, effectuer les smokes et garder provenance/reçus. Déploiement et rollback : [DEPLOYMENT](DEPLOYMENT.md). Incident : [RECOVERY](RECOVERY.md).

Une revue d'architecture est requise pour un changement de frontière de confiance, de format, de permission ou de compatibilité. Décrire options, migration et rollback dans un ADR/demande de changement avant de modifier un contrat gelé. Le propriétaire, les rôles architecture/sécurité et exploitation doivent être effectivement désignés ; le dépôt ne signe pas en leur nom.

Pour un défaut critique touchant un contrat gelé, suivre la partie G.2 du [plan](MASTER_PLAN.md#g2-contrats-gelés-implémentations-changeables-exceptions) : demande `docs/CHANGE_REQUESTS/CR-YYYY-NNN.md`, périmètre/impact/options/migration/tests/rollback/signataires, autorisations architecture + sécurité + propriétaire et branche `hotfix/CR-*`. Une autorisation de correction ne dispense pas de revérifier les étapes aval affectées. Aucun patch critique implicite ni gel V1 n'est accordé par ce guide.
