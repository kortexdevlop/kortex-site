# Hôte iframe — preuve 5.4

Le test tests/web/embed.spec.ts démarre deux serveurs localhost sur des ports distincts : catalogue parent et hôte d’exécution. Le parent sert un bouton d’ouverture, une iframe avec sandbox allow-scripts allow-same-origin, et un output pour RESULT. Le vrai shell Dioxus est servi dans frame.html.

La configuration de déploiement /embed-config.js fixe parent_origin, api_origin, clés publiques et descripteurs autorisés. Le fichier fourni est volontairement vide ; sans configuration hôte, le handshake n’est pas monté. Le parent ne choisit ni clés ni URL d’archive par postMessage.

L’entrée publique canonique est désormais /adapter/interop.js. Elle partage le jeton privé VerifiedTool et l’instance BrowserHost avec Rust. Une entrée host.js importée séparément peut être rebundlée par Dioxus et ne doit pas produire de jetons destinés à cette instance canonique.

Le serveur d’exécution doit installer les en-têtes produits par frameHeaders(parentOrigin) dans web-adapter/src/iframe.ts :

```
Content-Security-Policy: default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; worker-src 'self'; connect-src 'self'; style-src 'self'; style-src-attr 'unsafe-inline'; img-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-ancestors <origine-parent-fixe>
Referrer-Policy: no-referrer
X-Content-Type-Options: nosniff
Permissions-Policy: camera=(), microphone=(), geolocation=()
```

Le test ajoute un nonce propre à chaque réponse HTML aux scripts hôtes. Aucun unsafe-eval JavaScript. style-src-attr autorise les valeurs numériques du renderer contrôlé ; le guest ne fournit pas de CSS libre. frame-ancestors autorise exactement le catalogue. X-Frame-Options SAMEORIGIN/DENY contredirait cette intégration sur origine distincte ; la restriction d’ancêtres est assurée par CSP. Ces en-têtes sont des réponses HTTP, pas des balises meta.

Chromium et WebKit : vrai paquet signé ouvert ; Ready visible ; stockage et objet hôte inaccessibles au parent ; HELLO répété ignoré ; mauvaise version ignorée ; injection inline bloquée avec securitypolicyviolation ; parent non autorisé refusé ; Worker arrêté puis outil relancé. Le bouton du prototype parent mène à output=ready. Déploiement localhost prouvé, sans domaine public revendiqué.
