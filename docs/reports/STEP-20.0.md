# Étape 20.0 — Plan QA traçable

Clôture technique documentaire : `docs/QA_PLAN.md`, `docs/reports/QA_BASELINE.md` et `tests/e2e/README.md` fixent les environnements, commandes, données, métriques, seuils, exclusions et règles de preuve. Les 22 exigences I01–I22 ont un test ou une recette identifiée. Les validations finales non exécutées restent explicitement planifiées ou bloquées ; aucune signature humaine n’est créée.

Matrice composants × Windows x64 × Chromium/WebView2 × outils × permissions × réseau × identité, avec catalogue synthétique privé séparé des paquets signés réels. Linux, macOS, Firefox et WebKit exclus des nouveaux gates sur instruction du propriétaire. Lecteur d’écran manuel, Google réel, exploitation et production restent distincts des tests automatisés existants.

p95 défini par rang le plus proche sur 100 observations minimum, cinq échauffements exclus, corps HTTPS entièrement lu et reprises transport comprises dans la durée. Deux séries à 10 req/s ; liste ≤500 ms et détail ≤300 ms. Les échecs restent audités, les dépassements exigent un ADR, les tests ignorés doivent être justifiés.

Baseline distante réelle : D1/R2/Workers temporaires, Astro SSR existant déployé une fois, accès par secret et bail de deux heures. La preuve catalogue enregistre CPU, mémoire, Windows, Node, hashes des scripts, compteurs D1 observables et nettoyage. Les anciennes preuves CI sont datées et ne sont pas présentées comme une nouvelle exécution.

Validation locale `node scripts/ci-verify.mjs --lane contracts` réussie : registre, environnement, secrets, contrats, CSP, reprise et publication. Implémentation : 03becd5fd0614bafcfbeacaaf8ee677c0e74cb67, import fiabilisé b841c05904f56ba059be8f61a10bcece4d610ae4. Les gates finaux 20.2–20.5 et l’acceptation V1 ne sont pas réputés terminés.

Validation consolidée : [CI GitHub 40856245744](https://github.com/kortexdevlop/kortex-site/actions/runs/40856245744), source b841c05904f56ba059be8f61a10bcece4d610ae4, quatre lanes réussies (Windows natif, contrats, Web, sécurité). Preuve : `docs/evidence/qa-ci-validation.json`.
