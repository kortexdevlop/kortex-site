# Gouvernance et documentation — 21.3

Les règles de contribution, compatibilité, versioning, revue, exception critique et fin de vie sont réunies dans [CONTRIBUTING](../CONTRIBUTING.md) et [COMPATIBILITY_POLICY](../COMPATIBILITY_POLICY.md). La licence propriétaire choisie par le propriétaire est dans LICENSE ; les obligations des composants tiers restent à examiner avant distribution.

Le contrôleur `node scripts/check-doc-links.mjs` vérifie les cibles locales, les ancres Markdown et les références définies hors blocs d'exemple ; il ignore les URL externes et ne prétend pas tester leur disponibilité. Les chemins hors du dépôt et les références non portables sont refusés. Les noms de rôle/étape viennent de [FILE_OWNERS](../FILE_OWNERS.json) et les IDs sont contrôlés contre le plan. Chaque fichier versionné ou nouveau non ignoré doit avoir un responsable technique ; cela ne désigne pas une personne et ne fabrique pas son approbation.

Deux tests isolés du contrôleur couvrent cibles absentes, ancres invalides, sortie du dépôt, exemples et références Markdown. La preuve de la dernière exécution complète est `docs/evidence/documentation-gate.json` : zéro lien interne cassé et zéro fichier sans étape propriétaire requis avant clôture technique. Le registre est vérifié séparément.

Le guide du sixième outil reste la recette générique déjà éprouvée en 21.1 : build/validate/package, deux tests Base64 et sign/verify dry-run depuis clone propre, sans code spécifique dans les hôtes ni upload. Aucun golden ou contrat gelé n'a été réécrit ; les renvois de maintenance ont été ajoutés à l'historique des décisions/contrats.

Le gate humain global et la production restent distincts. Les tests de liens ne constituent pas une revue de sécurité ou de licence, et les pages sans référence visuelle ne sont pas annoncées pixel perfect.
