# Étape 19.2 — Déploiements API, site et Runtime

Les workflows API et Website sont séparés. Ils exigent la validation du même SHA, main et l’environnement staging protégé. L’API enregistre un point Time Travel D1 et applique les migrations uniquement en avant. Les paramètres vivants, secrets applicatifs et quotas sont conservés.

Website, Desktop et outils attendent une API compatible. X-Kortex-Api-Revision identifie la révision sans changer le DTO health V1. Une API antérieure n’est acceptée que si la comparaison des fichiers prouve l’absence de modification du périmètre API/DB/contrats.

Les exercices réels de retour à la version précédente puis restauration des Workers API et Runtime ont réussi. Le schéma D1 est resté identique. Aucune migration inverse ni restauration destructive de base n’a été réalisée. Voir pipeline-rollback-api.json et pipeline-rollback-runtime.json.

La publication Web/Desktop partage un verrou pour préserver le compteur signé et les deux descripteurs. DNS, messagerie et production sont conservés. Les runs [API 40817562928](https://github.com/kortexdevlop/kortex-site/actions/runs/40817562928) et [Website 40817566516](https://github.com/kortexdevlop/kortex-site/actions/runs/40817566516) sont réussis sur 53efb4e. Le rollback CI [40816034275](https://github.com/kortexdevlop/kortex-site/actions/runs/40816034275) a rétabli sa version courante, ses paramètres et le même schéma D1 ; voir pipeline-rollback-ci.json. La publication de Base64 est postérieure au dernier déploiement des hôtes et sa redelivery ne les a pas reconstruits : pipeline-redelivery.json.

Fichiers principaux :

- .github/workflows/api.yml
- .github/workflows/website.yml
- scripts/ci-deploy.mjs
- scripts/api-readiness.mjs
- scripts/rehearse-staging-rollback.mjs
- infra/deploy-order.md

Base du lot : 14aa8a4. Implémentation courante : 4950cbadb2a25a4ca377838f1cb54b4c2eeec397. Inventaire complet : pipeline-source-files.json. Le statut formel du registre ne vaut pas approbation humaine ; les critères techniques sont documentés séparément.

État technique final : **closed**, dans le périmètre Windows x64 / Chromium-WebView2 / staging autorisé. Les gates de production et la validation humaine formelle restent distincts.
