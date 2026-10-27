# KORTEX

Plateforme locale d’outils Rust Core WASM pour Web et Desktop. La calculatrice de référence fonctionne dans Chromium, WebKit et Windows/WebView2, avec le même paquet signé, un rendu partagé et quatre actions. Le catalogue et la publication cloud restent à construire.

Progression et preuves : [docs/PROGRESS.md](docs/PROGRESS.md). Plan source : [docs/MASTER_PLAN.md](docs/MASTER_PLAN.md). Création et vérification d’un outil : [docs/TOOL_SDK.md](docs/TOOL_SDK.md). Gate calculatrice : [docs/reports/PHASE7_GATE.md](docs/reports/PHASE7_GATE.md).

Le workspace utilise les versions figées de Rust, Node et pnpm. Installer les dépendances avec pnpm ; les suites Web exigent également la construction du shell Dioxus et les fixtures WASM indiquées dans les rapports. Les commandes de vérification de l’outil 1 et le scénario Windows sont dans le guide SDK. Linux/macOS et Firefox restent exclus sur instruction du propriétaire ; leur compatibilité n’est pas déclarée.

Configuration : copier .env.example vers .env pour le local ; ne jamais versionner les valeurs réelles. La signature des fixtures utilise une clé privée éphémère en mémoire, sans clé de production. Voir infra/environments.md pour les environnements prévus.
