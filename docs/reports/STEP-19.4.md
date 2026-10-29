# Étape 19.4 — Runtime Web et canaux signés

Runtime Web 1.0.1 est déployé sur son origine staging distincte. version.json décrit la compatibilité ABI/protocole/UI, le hash du bundle réellement construit et le bootstrap SRI. Le publisher compare ces valeurs avec la version déployée avant publication.

GET /api/v1/runtime/channels expose un document signé Ed25519 par une clé déléguée certifiée par la racine hors CI. Les hôtes Rust et TypeScript vérifient la même représentation canonique, l’expiration, le compteur monotone et la clé révoquée. stable reste nul.

ToolViewer attend la vérification du canal avant de créer l’iframe, puis contrôle l’ABI et min_runtime. Les tests communs couvrent altération, replay, expiration, ABI inconnue et UPDATE_REQUIRED. Le Desktop vérifie les mises à jour sans téléchargement ni installation automatiques et refuse le downgrade silencieux.

Vérifications observées : API 91 tests et Chromium 27 tests réussis ; ToolViewer staging charge le vrai paquet signé calculatrice et multiply(25,48) retourne 1200 ; WebView2 réel vérifie le canal. Le retour arrière staging a été exercé dans Chromium : le Runtime précédent a réellement chargé la calculatrice signée et retourné 1200, puis la version courante et ses paramètres ont été restaurés ; voir pipeline-rollback-runtime-browser.json. Voir pipeline-chromium.json, pipeline-live-viewer.json, pipeline-runtime-channels.json et pipeline-desktop-runtime-check.json.

Le canal final séquence 4 contient le Runtime Web 1.0.1 et Desktop Windows 1.0.0 publié. Le descripteur GitHub, sa provenance, le certificat et les signatures ont été vérifiés indépendamment ; le bootstrap servi correspond réellement au SRI annoncé et le proxy Website restitue la même signature. Voir pipeline-runtime-channels-final.json, pipeline-runtime-final-verification.json et pipeline-desktop-published.json. Le renouvellement des métadonnées (sept jours) et des révocations (24 h) reste une opération requise ; aucune clé racine privée n’est déplacée dans CI.

Fichiers principaux :

- runtime/web/worker.ts
- runtime/web/public/version.json
- scripts/build-runtime.mjs
- scripts/publish-runtime.mjs
- scripts/runtime-publication.mjs
- shared/typescript/src/runtime.ts
- api/src/routes/runtime.ts
- crates/kortex-verify/src/runtime.rs
- desktop/src/runtime_update.rs
- website/src/components/ToolViewer.astro
- tests/integration/runtime-update.spec.ts

Base du lot : 14aa8a4. Implémentation courante : 4950cbadb2a25a4ca377838f1cb54b4c2eeec397. Inventaire complet : pipeline-source-files.json. Le statut formel du registre ne vaut pas approbation humaine ; les critères techniques sont documentés séparément.

État technique final : **closed**, dans le périmètre Windows x64 / Chromium-WebView2 / staging autorisé. Les gates de production et la validation humaine formelle restent distincts.

Le schéma d’outil 1.0 figé conserve min_runtime 1.0.0. Le garde de compatibilité et UPDATE_REQUIRED sont éprouvés dans le contrat partagé et dans Chromium ; aucun paquet V1 avec un min_runtime non conforme au schéma figé n’est présenté comme publié. Les goldens de paquets et l’ABI restent inchangés.
