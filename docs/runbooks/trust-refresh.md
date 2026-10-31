# Renouvellement des documents de confiance

La liste de révocation signée est valable 24 h (contrat) et le canal Runtime 30 jours au plus. Sans renouvellement, l'API répond « Signed revocations unavailable » et les outils ne s'ouvrent plus.

- Commande : `node scripts/refresh-trust.mjs --env production` (ou `staging`, `--force` pour forcer).
- Automatisation : tâche planifiée Windows « Kortex trust refresh » (toutes les 4 h, session de l'opérateur, machine allumée, connexion wrangler « kortex » active). Suppression : `Unregister-ScheduledTask -TaskName 'Kortex trust refresh' -Confirm:$false`.
- Les clés racines restent chiffrées par DPAPI dans ~/.kortex-offline ; rien n'est exporté.
- Journal : target/trust-refresh.log. Redéployer l'API avec scripts/production-release.mjs conserve les derniers documents (target/*-latest.json).
