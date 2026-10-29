# Étape 19.0 — Validation GitHub

Périmètre : Windows x64 et Chromium, conformément aux exclusions demandées. Le dépôt public kortexdevlop/kortex-site exécute quatre jobs indépendants en lecture seule : contrats, Rust, Web et sécurité. Les actions et outils sont épinglés, les lockfiles sont obligatoires et les rapports sont archivés sept jours.

Les fixtures sont préparées sur un checkout neuf. Les références binaires historiques restent inchangées ; une reconstruction courante de la calculatrice exécute en plus les mêmes goldens métier. Les sources générées ne dépendent pas d’un ancien target local.

Preuve négative réelle : la PR [14](https://github.com/kortexdevlop/kortex-site/pull/14) a échoué sur le contrat altéré. Elle est fermée sans fusion. Le run ne recevait aucun secret de publication. Voir pipeline-negative-pr.json.

Validation locale finale : 91 tests API et 27 scénarios Chromium réussis ; typage API réussi. Les suites Rust complètes sont vérifiées sur le runner Windows pour éviter de confondre l’échec local de disque plein avec un résultat fonctionnel.

Le run [40844352974](https://github.com/kortexdevlop/kortex-site/actions/runs/40844352974), commit 4950cbadb2a25a4ca377838f1cb54b4c2eeec397, est entièrement vert : quatre jobs réussis, 188 tests Rust réussis, 91 API et 27 Chromium. Un smoke HTTPS externe explicitement optionnel est ignoré dans cargo test ; ce résultat n’est pas présenté comme une réussite. Voir pipeline-ci-validation.json. Les gates de production de 18.2/18.4 ne sont pas réputés levés.

Fichiers principaux :

- .github/workflows/validate.yml
- scripts/ci-prepare.mjs
- scripts/ci-verify.mjs
- tests/fixtures/
- scripts/security-gate.mjs
- docs/CI.md

Base du lot : 14aa8a4. Implémentation courante : 4950cbadb2a25a4ca377838f1cb54b4c2eeec397. Inventaire complet : pipeline-source-files.json. Le statut formel du registre ne vaut pas approbation humaine ; les critères techniques sont documentés séparément.

État technique final : **closed**, dans le périmètre Windows x64 / Chromium-WebView2 / staging autorisé. Les gates de production et la validation humaine formelle restent distincts.
