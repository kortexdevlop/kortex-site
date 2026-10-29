# Étape 19.1 — Publication des outils

Le workflow tools.yml détecte les seuls dossiers outils modifiés et construit une matrice ciblée. Chaque paquet passe les tests et la validation, est signé par la clé déléguée protégée, envoyé dans R2 sous un nom adressé par SHA-256 puis publié via l’API admin. La signature appelle le binaire xtask déjà construit sans relancer Cargo avec la clé privée. Les enfants de compilation/vérification ne reçoivent pas les identifiants.

L’API vérifie la signature RSA OIDC GitHub et les claims dépôt, ID numérique, branche, environnement, audience et workflow exacts. Le sujet utilise les identifiants immuables de compte/dépôt exposés par GitHub. Le format x5t documenté par GitHub est couvert par un test réel RSA ; les extensions non reconnues sont refusées. La durée du jeton reste bornée à dix minutes.

Le sixième outil réel encodeur-base64 est indépendant des hôtes. L’identifiant de requête est dérivé des octets ; les reprises sont bornées et ne remplacent pas un objet différent. La provenance signée n’est pas présentée comme une certification SLSA.

Le transport réel R2 PUT/GET/hash/DELETE a réussi. La publication [40817599252](https://github.com/kortexdevlop/kortex-site/actions/runs/40817599252) et la relivraison [40839663529](https://github.com/kortexdevlop/kortex-site/actions/runs/40839663529) sont réussies. L’API sert encodeur-base64 1.0.0, hash 75637ad92cea6ffd21be272e53f2fdc485265c6c2a94c14406b6f0bf8b699139. Une seule publication est enregistrée en D1 ; hash, signature et date sont inchangés après redelivery. Les Worker Website/Runtime et le bundle sont inchangés. Chromium a réellement saisi du texte UTF-8 et vérifié le résultat local ; encode(abc) retourne YWJj. Voir pipeline-base64-published.json, pipeline-base64-browser.json et pipeline-redelivery.json. La production reste non configurée et refusée.

Fichiers principaux :

- .github/workflows/tools.yml
- scripts/ci-publish-tool.mjs
- scripts/release-tool.mjs
- api/src/security/admin-auth.ts
- tools/encodeur-base64/
- tests/publication/

Base du lot : 14aa8a4. Implémentation courante : 4950cbadb2a25a4ca377838f1cb54b4c2eeec397. Inventaire complet : pipeline-source-files.json. Le statut formel du registre ne vaut pas approbation humaine ; les critères techniques sont documentés séparément.

Références du diagnostic : [OIDC GitHub](https://docs.github.com/en/actions/concepts/security/openid-connect) et [sujet immuable](https://docs.github.com/en/rest/actions/oidc). La politique effective et les identifiants publics observés sont dans pipeline-github-settings.json.

État technique final : **closed**, dans le périmètre Windows x64 / Chromium-WebView2 / staging autorisé. Les gates de production et la validation humaine formelle restent distincts.
