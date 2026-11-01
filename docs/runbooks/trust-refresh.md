# Renouvellement des documents de confiance

La liste de révocation signée est valable 24 h (contrat) et le canal Runtime 30 jours au plus. Sans document valide, l'API répond « Signed revocations unavailable » et les outils ne s'ouvrent plus.

## Production : documents signés à l'avance

`node scripts/refresh-trust.mjs --env production` signe 90 jours de listes (une toutes les 12 h) et de canaux (un toutes les 2 semaines) avec les clés hors ligne, puis les dépose dans le bucket privé (`trust/schedule.json`). L'API ne détient aucune clé : elle choisit le document valide et vérifie sa signature. Le PC n'a donc besoin de tourner que tous les 45 jours environ ; la tâche planifiée « Kortex trust refresh » reconstruit la série quand il reste moins de 45 jours.

- **Après une révocation :** lancer `node scripts/refresh-trust.mjs --env production --force`. Tant que ce n'est pas fait, l'API refuse les documents qui ne contiennent pas la révocation et répond 503 dès l'expiration du document courant : c'est volontaire.
- **Après une publication du Runtime :** `scripts/production-release.mjs deploy` reconstruit la série automatiquement.
- **Limite :** le certificat des clés expire le 31 octobre 2028. Avant cette date, lancer `node scripts/renew-production-keys.mjs`, ajouter la ligne SQL affichée à la table `signing_keys`, déployer, puis publier un nouvel installateur : les applications déjà installées gardent leur certificat intégré.

## Staging

La même commande avec `--env staging` renouvelle directement les documents, sans série. La CI web dépend de ces documents.

Les clés racines restent chiffrées par DPAPI dans ~/.kortex-offline ; rien n'est exporté. Journal : target/trust-refresh.log. Suppression de la tâche : `Unregister-ScheduledTask -TaskName 'Kortex trust refresh' -Confirm:$false`.
