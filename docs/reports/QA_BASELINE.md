# Baseline QA — 29 octobre 2026

Périmètre : Windows x64, Chromium/WebView2, staging. Versions épinglées : Node 24.15.0 (moteur >=24.15 <25), pnpm 11.25.0, Rust 1.99.1, Wrangler 4.152.0, Playwright 1.64.1, Vitest 5.0.5. Les versions effectives de la machine et son CPU sont enregistrés par le rapport de charge ; le runner CI peut évoluer indépendamment du poste Windows local.

Référence précédente reproductible : source 4950cbadb2a25a4ca377838f1cb54b4c2eeec397, CI Validate 40844352974 et livraison Desktop 40844353018 ; 188 Rust, 91 API et 27 Chromium, un smoke HTTPS facultatif justifié. Installateur x64 testé sur Windows Server 2022 frais, WebView2 vérifié. Ces chiffres sont historiques et ne sont pas attribués au présent commit sans une nouvelle CI.

Charge distante : génération d’un nom propriétaire aléatoire `kortex-qa-<nonce>`, nouvelle D1 avec toutes les migrations, bucket distinct, API réelle compilée, Astro SSR réel déjà compilé puis déployé une fois. Accès HTTP protégé par secret aléatoire et bail de deux heures, absence de service IA et de comptes réels. Les secrets passent par les bindings secrets Cloudflare et la mémoire, jamais par les preuves. Les clés de signature de test expirent avec la suppression de l’environnement ; elles ne remplacent aucune racine réelle.

Les fixtures 10k sont des métadonnées marquées `SYNTHETIC_METADATA_NO_R2`. Elles traversent les validateurs catalogue de production mais ne constituent ni une publication cryptographique réelle ni des outils exécutables. Le drill release distinct utilise un véritable paquet canonique avec le WASM calculatrice figé, une signature Ed25519 vérifiée et de vrais octets R2.

Ressources partagées exclues : catalogue staging réel, production, DNS, messagerie, utilisateurs, secrets applicatifs et quota Workers AI. Aucun changement de plan payant ni restauration D1 destructive. Le script supprime exclusivement ses propres noms et vérifie le marqueur propriétaire avant suppression des Workers et le nom D1 avant suppression de la base.

La durée réseau dépend du poste et de la région Cloudflare. Les résultats ne sont pas des garanties de production. Voir `CATALOG_SCALE_10K.md` et les observations JSON pour les percentiles, le stockage, les index et les ressources réellement utilisées.
