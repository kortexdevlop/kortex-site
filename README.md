# KORTEX

Plateforme d’outils Rust Core WASM exécutés localement dans le navigateur ou l’application Windows. Le catalogue dynamique et les archives signées sont servis par Cloudflare D1/R2/API ; le Runtime isolé vérifie les paquets avant exécution. Le périmètre actuel de validation est Windows x64 et Chromium/WebView2. Linux/macOS et Firefox sont exclus sur instruction du propriétaire.

Le [site de staging](https://kortex-web-staging.userpgl.workers.dev/workspace), son [API](https://kortex-api-staging.userpgl.workers.dev/api/v1/health) et le [Runtime](https://kortex-run-staging.userpgl.workers.dev/version.json) sont séparés. La production n’est pas configurée.

Progression et preuves : [registre](docs/PROGRESS.md), [rapports](docs/reports/), [plan source](docs/MASTER_PLAN.md). Création et vérification d’un outil : [guide SDK](docs/TOOL_SDK.md). Construction et validation : [CI](docs/CI.md), [distribution Windows](docs/DESKTOP_DISTRIBUTION.md), [ordre de déploiement](infra/deploy-order.md).

Installer les dépendances avec pnpm et les versions verrouillées du dépôt. La validation complète passe par node scripts/ci-verify.mjs ; elle prépare ses propres fixtures et exige les outils décrits dans docs/DEPENDENCY_BASELINE.md. Les clés privées de test sont éphémères ; les clés de confiance versionnées sont publiques. Les identifiants réels restent hors Git.

## Canaux du Runtime

GET /api/v1/runtime/channels fournit les canaux signés stable et staging. Le canal staging référence le Runtime Web 1.0.1, son origine, le hash du bundle, le bootstrap SRI et l’installateur Windows lorsqu’il est publié. stable reste nul jusqu’à l’approbation de production. Les préversions Windows de staging sont explicitement non signées Authenticode.

Web et Desktop vérifient la signature, l’expiration et le compteur croissant avant admission. Un outil exigeant un Runtime plus récent reçoit UPDATE_REQUIRED ; une ABI inconnue est refusée. Aucun téléchargement ni downgrade Desktop n’est automatique. Un retour arrière du Worker staging est une opération explicite qui conserve la compatibilité DB et restaure les paramètres courants ; il ne contourne pas le compteur de confiance d’un client.

Les métadonnées de canal expirent après sept jours au maximum dans le publisher actuel. Les révocations expirent après vingt-quatre heures et nécessitent un renouvellement opérateur signé hors CI ; une révocation connue reste bloquante même après expiration. Voir [distribution et révocations](docs/DISTRIBUTION_CI.md).

Préversion Windows disponible : [Kortex Desktop 1.0.0](https://github.com/kortexdevlop/kortex-site/releases/tag/desktop-staging-v1.0.0). Cet installateur staging est non signé Authenticode ; son checksum et sa provenance signée sont publiés avec la release.
