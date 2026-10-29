# Étape 19.3 — Installateur Desktop Windows

Le périmètre Windows x64 est explicitement demandé par le propriétaire ; Linux/macOS et Firefox sont exclus. Aucune réussite sur ces plateformes n’est déclarée.

Dioxus CLI 0.7.10 produit un véritable EXE NSIS 3.11 avec WebView2 hors ligne. Le wrapper gère le défaut d’encodage NSIS observé sur le chemin µ en reprenant le même script généré en UTF-8. Les versions, commandes et sources sont documentées dans DESKTOP_DISTRIBUTION.md et DEPENDENCY_BASELINE.md.

L’installateur local a été installé silencieusement dans un dossier neuf, démarré puis désinstallé ; la disparition du binaire a été vérifiée. Un contrôle distinct du WebView2 réel confirme la consultation des canaux et la signature des métadonnées. Le contrôle natif final lit le canal signé séquence 4. Voir pipeline-desktop-x64-package.json, pipeline-desktop-ci-smoke.json, pipeline-desktop-runtime-check.json et pipeline-desktop-version.png.

Le workflow reconstruit et refait le smoke sur Windows, puis publie une préversion GitHub immuable avec checksum et provenance signée avant d’actualiser le canal Desktop. Le run [40844353018](https://github.com/kortexdevlop/kortex-site/actions/runs/40844353018) est réussi, compilation puis installation/démarrage/désinstallation sur une machine Windows Server 2022 fraîche, enfin publication protégée. Le Runtime installé est 156.0.4314.50 sans remplacement par un navigateur Edge. Voir pipeline-desktop-ci-smoke.json.

Limite explicite : l’EXE staging est non signé Authenticode. La provenance signée ne remplace pas le certificat Windows. La production reste bloquée sans certificat réel et sans levée des gates de lancement.

Fichiers principaux :

- .github/workflows/desktop.yml
- scripts/desktop-package.mjs
- scripts/desktop-installer-smoke.ps1
- scripts/publish-desktop.mjs
- desktop/Dioxus.toml
- tests/desktop/

Base du lot : 14aa8a4. Implémentation courante : 4950cbadb2a25a4ca377838f1cb54b4c2eeec397. Inventaire complet : pipeline-source-files.json. Le statut formel du registre ne vaut pas approbation humaine ; les critères techniques sont documentés séparément.

État technique final : **closed**, dans le périmètre Windows x64 / Chromium-WebView2 / staging autorisé. Les gates de production et la validation humaine formelle restent distincts.

Préversion publiée : [Desktop Windows 1.0.0](https://github.com/kortexdevlop/kortex-site/releases/tag/desktop-staging-v1.0.0). L’EXE CI fait 225520032 octets ; SHA-256 f73a0a4c74376b7d741d04fc06bf67ffacce001324a05aadd0846bbdf30ef9ac. Le digest GitHub, la provenance signée et le canal public correspondent ; voir pipeline-desktop-published.json et pipeline-desktop-provenance.json. Les builds locaux et les essais antérieurs restent identifiés séparément et ne constituent pas cette release immuable.

Défaut réel corrigé : Dioxus CLI 0.7.10 utilisait le lien x86 2099617 pour toutes les architectures. Le packageur précharge désormais le fichier x64 du lien Microsoft 2124701, avec URL CDN, taille, SHA-256 et signature Microsoft vérifiés. Aucun script tiers du registre Cargo n’est modifié. Voir infra/desktop/webview2-x64.json, scripts/desktop-webview2.mjs et pipeline-desktop-x64-package.json. Les tests refusent l’ancien cache, une architecture incorrecte, une empreinte altérée ou un éditeur non valide.
