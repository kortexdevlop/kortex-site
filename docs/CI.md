# CI Kortex

Le dépôt est public sur autorisation du propriétaire : https://github.com/kortexdevlop/kortex-site. Les erreurs de facturation du dépôt privé sont historiques ; les runners publics exécutent désormais les jobs.

`validate.yml` exécute quatre jobs Windows indépendants : contrats/goldens, Rust fmt/clippy/tests workspace, Web types/tests/build/sécurité Chromium, audits et SBOM. Chaque job est en lecture seule. Les PR n'ont aucun secret de staging et ne publient rien. Les sources générées API, adaptateur et site sont préparées explicitement sur un runner neuf ; les anciens dossiers target ne constituent pas une dépendance.

Les références JSON/NDJSON et les hash de la calculatrice restent inchangés. `tests/fixtures/core-reference.wasm` et `tests/fixtures/calculatrice-reference.wasm` conservent exactement les binaires référencés par leurs hash figés, pour le contrôle de régression byte à byte. En parallèle, le compilateur reconstruit la calculatrice courante et Chromium exécute le même corpus métier sur ce nouveau module signé. Les chemins du runner peuvent modifier le hash de reconstruction ; cela ne remplace jamais le contrôle du binaire figé. Les SVG utilisés dans les archives sont en LF pour éviter une dérive CRLF du packaging Windows.

Les actions sont référencées par commit. Le cache Cargo ne contient que les sources du registre ; aucun exécutable de publication préparé sur une PR n'est réutilisé avec une clé privée. Les rapports sont archivés sept jours. Les clés privées et tokens ne font pas partie des rapports ou artefacts.

Les publications et déploiements exigent une validation réussie du **même SHA** sur main, puis l'environnement staging protégé. L'API vérifie aussi l'identité OIDC GitHub exacte du workflow tools.yml. Les secrets de staging comprennent le token Cloudflare et la clé de release déléguée ; la racine privée reste sous DPAPI chez l'opérateur. Production exige une approbation du propriétaire et reste bloquée dans les scripts tant que les gates de lancement ne sont pas levés.

`tools.yml` détecte les seuls dossiers outils modifiés ; build, tests, validation et packaging sont ciblés. La publication R2 est immuable, la requête idempotente, les reprises bornées et la provenance signée. Ajouter un dossier sous tools et mettre le lock à jour ne demande aucun branchement spécifique dans les hôtes.

`api.yml` et `website.yml` sont indépendants. L'option manuelle `rehearse_rollback` exerce un retour arrière staging puis restaure la version courante et ses paramètres ; aucune base de données n'est restaurée par ce test. Voir `infra/deploy-order.md`.

`desktop.yml` fabrique l'installateur Windows, l'installe dans un dossier de test neuf, démarre le processus, le désinstalle et publie une préversion staging avec checksum et provenance. Linux/macOS/Firefox sont exclus sur instruction du propriétaire. La signature Authenticode de production nécessite un certificat réel ; la signature de provenance ne la remplace pas.

Les processus de compilation et vérification ne reçoivent pas les tokens Cloudflare/GitHub ni les autres secrets de l’environnement. La signature appelle directement le binaire xtask déjà compilé, avec la seule clé de signature nécessaire ; elle ne relance pas Cargo avec la clé privée.
