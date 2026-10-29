# Restaurer D1 sans toucher à la production pendant le drill

Commencer par la [procédure générale](../RECOVERY.md). Vérifier compte, nom et UUID : la base temporaire du drill doit être distincte de `infra/deploy/channels.json` pour staging et production. Ne pas restaurer la base servant les utilisateurs pour prouver une procédure.

## État et point de retour

Suspendre les écritures/publications de la cible. Exporter et protéger l'état actuel selon les accès disponibles. Relever le bookmark actuel avant toute restauration, les migrations appliquées, les reçus et les versions de code. Vérifier `version: production` avec la commande `d1 info` de Wrangler. Le quota D1 gratuit reste partagé au compte, même sur une autre base.

Les formes réelles du CLI sont, avec une configuration relue pointant exclusivement sur la cible autorisée :

```powershell
pnpm.cmd exec wrangler d1 info DB --config <configuration-validée>
pnpm.cmd exec wrangler d1 time-travel info DB --config <configuration-validée> --json
pnpm.cmd exec wrangler d1 export DB --remote --config <configuration-validée> --output <sauvegarde-locale-protégée.sql>
```

Les chevrons sont des valeurs à renseigner, pas un exemple exécutable. L'export peut contenir les données de compte : il ne doit pas être commité ou joint à une issue. Le bookmark n'est pas une copie SQL et ne quitte pas la frontière du compte.

## Restaurer et garder une issue de secours

Choisir un bookmark antérieur à l'incident dans la fenêtre disponible (7 jours Free). Lire l'impact, puis utiliser `wrangler d1 time-travel restore DB --bookmark <bookmark-validé> --config <configuration-validée>`. Les requêtes en cours peuvent être annulées. Conserver le `previous_bookmark` renvoyé : il permet de revenir à l'état qui précédait la restauration. Un retour de l'API fournisseur ne remplace pas la vérification du contenu et des services.

Comparer le catalogue, les migrations, les références d'archives et les contraintes ; vérifier les suppressions/tombstones, clés retirées et sessions révoquées après le point récupéré. Ne pas rouvrir le trafic en restaurant aveuglément d'anciens droits. Réappliquer les retraits de sécurité à partir d'une source fiable sans diminuer les compteurs de confiance.

Rétablir séparément les objets R2 manquants depuis la copie vérifiée. Tester API, site et téléchargement, puis les actions réelles. En cas de mauvais choix de point, l'opération inverse utilise le bookmark précédent après la même revue de cible et d'impact.

## Drill du dépôt

`node scripts/qa-disaster-drill.mjs --isolated-cloud` crée ses propres DB/bucket/Workers privés, limite les données à un outil et les restaurations à trois, puis vérifie leur propriétaire avant nettoyage. Il ne restaure jamais la DB staging. Il s'arrête sur un quota ; il ne lève pas la limite et n'achète aucun forfait. Le dernier essai est dans [DISASTER_DRILL](../reports/DISASTER_DRILL.md).

`node scripts/qa-local-restore.mjs`, après un nouveau `qa-local-system.mjs`, vérifie le chemin de sauvegarde SQLite et les octets R2 dans l'émulation locale. Cette recette distincte n'est pas un test Cloudflare Time Travel et ne prouve pas la disponibilité du compte distant.
