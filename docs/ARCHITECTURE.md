# Architecture Kortex — état vérifié à la revue 3.4

**Lot site 11.1–11.3 (27/10/2026) :** catalogue/fiches SSR réels, accueil workspace alimenté par l’API et shell /app avec pont HELLO/READY/OPEN existant. Vérifiés sur Worker compilé, D1 locale et véritable Runtime signé (1200), sans déploiement. État et limites : docs/reports/STEP-11.1.md à STEP-11.3.md. Les états anciens ci-dessous restent historiques.

**Direction visuelle active (27/10/2026) :** la capture Kortex Workspace-selection.png fournie par le propriétaire remplace le concept initial. Utiliser WorkspaceLayout/Workspace pour les pages publiques ; l’ancien Header horizontal et les cartes pastel sont gardés uniquement dans l’atelier de compatibilité. Mesures, référence immuable et limites dans docs/DESIGN.md et docs/reports/DESIGN-WORKSPACE-2026-10-27.md. Les descriptions visuelles antérieures ci-dessous sont historiques. Aucun contrat runtime/API gelé n’est modifié.

23 octobre 2026. Référence de conception complète : MASTER_PLAN.md parties A–D, conservée à l'identique. Cette page distingue les preuves locales de la cible à construire. Décision initiale : NO-GO pour gel et phase 4 jusqu'à résolution de CR-2026-001 ; voir evidence/ARCHITECTURE_GATE.md et adr/0007-prototype-outcome.md.

## Faits établis

- Le guest Rust Core WASM cible wasm32-unknown-unknown. Les mêmes octets de 3.0 ont été exécutés dans Chromium/WebKit et Wasmtime Windows ; 30 réponses complètes identiques, hash conservé dans les preuves 3.0/3.1.
- L'ABI expérimentale expose memory, kortex_alloc, kortex_dealloc, kortex_dispatch et importe kortex_v1.host_enqueue. Aucun WASI, DOM ou accès natif libre. Appel host refusé dans les bancs sans permissions. Queue asynchrone avec résultats positifs non démontrée.
- Le renderer Dioxus 0.7.10 interprète le sous-ensemble du fixture dans le Web et une WebView2 Desktop Windows. Label, saisie, clic, focus, graphique SVG et captures vérifiés. Table et carte hôtes supplémentaires ; renderer complet et dispatch d'événements vers guest à implémenter.
- Iframe parent/runtime sur deux ports locaux : same-origin policy, origine/source/nonce, CSP et Worker testés dans Chromium/WebKit. Validation ZIP hostile sous Node, avec clé de test éphémère ; parseur produit dans le navigateur à construire.
- Les trois divergences codec/schéma découvertes au gate 3.4 sont corrigées par CR-2026-001 ; 46 cas croisés et nouveaux essais Web/native réussissent. La réussite des prototypes ne prouve pas une conformité totale.

## Cible restant à construire

API Worker comme source métier unique, catalogue D1, artefacts immuables R2, Astro SSR via Service Binding, runtimes Web/Desktop Dioxus, médiation des capacités, consentement local, workflows et IA proposant des actions. Rien de ce catalogue, de l'identité OIDC, de l'IA ou des services cloud n'est présenté comme déployé.

La distribution conservera SHA-256 et signature détachée Ed25519 ; les clés de confiance persistantes et leur rotation signée restent à mettre en œuvre. Les outils ne s'appellent pas directement, orchestration par l'hôte. Simple/Medium/Pro reste une densité visuelle, sans plan payant. Les versions courantes seront basculées transactionnellement dans le catalogue ; comportement non encore exercé à cette date.

Les limites candidates sont dans CONTRACTS.md et les schémas. Aucun contrat FROZEN_V1. La baseline consigne Dioxus/dx, Rust, Wasmtime, Node et les dépendances exactes ; feature web-sys/Location nécessaire au prototype Web. Le futur runtime devra revalider ces choix sous ses propres contraintes.

## Périmètre humain accepté et limites

Linux/macOS omis sur instruction du propriétaire ; Firefox omis après échec de lancement et instruction explicite. Ces systèmes ne sont pas déclarés compatibles. Protection de branche GitHub distante non activée ; poursuite locale autorisée par le propriétaire. Aucune signature ou attestation technique/sécurité n'est inventée. Le gel exige une nouvelle approbation après correction et preuves.

Reprise de revue 3.4 après CR-2026-001 : les trois divergences de codec sont corrigées et les preuves Web/native rejouées sur un nouveau hash identifié. La recommandation technique passe à GO proposé sur le périmètre local démontré, sous réserve de revue humaine du gate. Aucun gel automatique ni compatibilité supplémentaire déclarée. Le NO-GO décrit plus haut est le constat initial, conservé pour traçabilité.

## Outil de référence : calculatrice (déclaration 7.0)

Le manifeste et le contrat de tools/calculatrice déclarent quatre actions pure compute (add, subtract, multiply, divide), sans permission sensible. Le design et les cas numériques sont versionnés avec le contrat. Les clics et action.invoke partageront les mêmes fonctions guest en 7.1 ; aucun identifiant d’outil ne doit déclencher une branche spéciale dans les hôtes. Le code de 7.0 contient uniquement les déclarations, sans export ABI ni paquet exécutable.

## Backend natif — 6.2 et 6.3

Un import host_enqueue borné collecte les demandes SDK du guest signé. L’identité et les déclarations viennent du Core ; Gateway et les consentements hôtes précèdent les effets. Les fichiers sont conservés ouverts par handles opaques, le stockage non secret est résolu sous le répertoire local OS et le réseau HTTPS connecte exclusivement les IP publiques validées d’origines hôtes autorisées. Aucun WASI ni lancement de processus guest. Fermeture, changement de génération et annulation révoquent la session. Les résultats reviennent par capability.result puis render via le Core. La vue Desktop et les IO attendent sans réveil actif.

## Calculatrice de référence — 7.1 à 7.3

Le trait Tool et les exports SDK de tools/calculatrice exécutent maintenant les quatre actions. Les clics partagent State::calculate avec action.invoke ; l’état de saisie et les erreurs appartiennent au guest. Le même ZIP signé local est admis par ToolLoader/TrustStore côté Web et verify_package côté natif. Le guest a zéro import ; les deux hôtes rendent son arbre UI v1 et le ferment sans persistance des calculs.

Un défaut transversal préexistant du pont Rust→JavaScript Web a été corrigé : les commandes doivent devenir des objets JavaScript ordinaires, pas des Map. Les clics réels du parcours calculatrice servent de régression. La CSS commune assure aussi une largeur minimale de bouton de 44 px, en complément de la hauteur existante. Aucun traitement du nom calculatrice, changement de moteur ou nouveau schéma n’a été nécessaire. Les goldens Request/Reply et les empreintes sont sous tests/golden ; voir TOOL_SDK.md et reports/PHASE7_GATE.md.

Cette section remplace l’état « déclaration seule » de 7.0 ci-dessus, conservé comme historique. Entrée/Espace activent les boutons natifs focalisés ; le protocole figé ne transporte pas les raccourcis globaux Entrée/Échap envisagés dans le premier design. Ils ne sont pas revendiqués.

## Packageur et signature — 8.0 à 8.2

Les crates kortex-pack, kortex-xtask et kortex-verify livrent le ZIP canonique, les commandes génériques et la vérification SHA-256/Ed25519 B.2. Le scanner utilise les manifestes et Cargo metadata, sans branche par identifiant. Le validateur effectue une inspection ZIP avant décompression bornée en mémoire et une validation statique du WASM. La signature est détachée, avec graine injectée seulement dans le processus CI ; la racine hors ligne ne publie que son manifeste de clés signé.

Le vérificateur Rust et TrustStore Web vérifient la même fixture publique calculatrice. Le snapshot de clés versionné contrôle dates, révocation et minimum de version fourni par l’hôte. Aucune ancre de production, distribution de révocation distante ou publication cloud n’est configurée. Le CLI garde un profil de confiance TEST explicite par défaut ; la migration des anciens chemins d’admission natifs vers un snapshot distribué sera vérifiée aux étapes responsables. Les limites de publication du nouveau packager sont plus strictes que les prototypes d’admission historiques et ne prétendent pas les avoir tous remplacés.

Documentation : SIGNING.md. Les cinq schémas gelés et la référence TOOL1_V1 sont conservés. Le nouveau schéma artifact et le format canonique restent candidats jusqu’à 8.5 ; le lot présent s’arrête à 8.2.

## Distribution locale et catalogue — 8.3 à 9.0

PACKAGE_FORMAT_V1 et le descripteur sont maintenant gelés par le gate local 8.5 ; les paragraphes candidats précédents restent historiques. Le pipeline release-tool prépare l'artefact signé et PublicationRequest sous target/, sans réseau cloud. Web et Windows vérifient le hash/signature avant lecture du ZIP, puis métadonnées, contenu autorisé et ABI statique avant création de l'instance. Windows accepte la configuration hôte signée par racine ; l'ancien jeu de clés publiques épinglées reste compatible. Le guest ne fournit pas son ancre. La version minimale du snapshot demeure fournie par l'hôte, sans service de révocation distant revendiqué.

La migration 0001 crée catégories, outils, versions, actions et journal de publication, sans binaires. Les quatre index de pagination servent les deux tris stables avec/sans catégorie sur les seuls outils publiés. Une identité/version est immuable ; le pointeur courant référence une version publiée du même outil. Les 10 000 outils de test sont synthétiques, pas 10 000 binaires réalisés. DATABASE.md décrit contraintes, index et migrations ; le binding KORTEX_DB est prévu, non provisionné. La publication atomique R2/D1 et l'API ne sont pas encore implémentées.

## État de publication et API locale — 9.1 / préparation 9.2

0002 ajoute clés publiques, artefacts immuables, jobs, audit des transitions et révocations. Le service devra vérifier réellement R2 et les signatures avant d'écrire les attestations exigées par le SQL ; la base n'exécute pas ces vérifications externes. Une révocation retire d'abord les pointeurs actifs dans la transaction ; les états ne se réactivent pas. Un rollback de catalogue peut sélectionner une ancienne version encore autorisée. Politique : infra/r2-policy.md.

Le Worker API local expose la liveness /api/v1/health et les erreurs versionnées, avec DB et TOOLS_BUCKET locaux, sans AI. Le nom DB remplace le nom de binding provisoire KORTEX_DB pour l'application ; le banc historique garde son nom interne. La configuration est validée au runtime sans exposer les secrets. Le catalogue/recherche/publication HTTP restent aux étapes suivantes. api.schema.json est candidat, sans modification des six schémas gelés. La préparation locale 9.2 passe ; la validation distante s'arrête à la demande du propriétaire sur les autorisations OAuth insuffisantes. Aucun provisionnement ou déploiement. Guide de reprise : infra/cloudflare-setup.md.
# Design system partagé — préparation 10.0–10.2

Le site local Astro 7.4.2 utilise Tailwind 4.3.3 (plugin Vite) et des composants `.astro` réutilisables. Source unique `website/src/styles/tokens.json` → générateur `scripts/build-design-tokens.mjs` → `theme.css` ; `global.css` consomme les variables. Les thèmes clair/sombre, densités gratuites simple/medium/pro et taille du texte s’appliquent sur `<html>` ; le shell contrôle les chaînes FR/EN. Préférences locales versionnées par la clé `kortex-display`, sans appel réseau et avec repli si le stockage est refusé.

MainLayout compose Header/Footer ; les primitives Button/Pill/SearchField partagent les styles et les états accessibles. Le Header prépare les destinations produit sans figer le routage. L’atelier `/__dev/design` est injecté seulement en développement depuis `src/previews`, hors de l’arbre de pages : il n’est pas publié. La configuration reste statique et locale jusqu’à 11.0. Tests tokens/contrastes et Chromium/WebKit séparés des tests runtime/API existants, serveur local contrôlé uniquement pour `tests/visual` (ou `KORTEX_DESIGN_TEST=1`).

La référence 1632 × 927 manque : les styles sont candidats, sans mesure ou fidélité visuelle attestée. Voir `docs/DESIGN.md` et `website/src/assets/reference-annotations.md`. Aucun contrat gelé ni ressource Cloudflare n’est modifié par ce lot.

## Actualisation 11.0 — site SSR local

Astro passe à output server avec adaptateur Cloudflare 14.4.2, Tailwind conservé, Preact comme unique framework d’îlots (ADR-0011). Les variantes SSR et navigateur utilisent ApiClient.listTools/getTool/getActions, réponses contrôlées par les validateurs générés des schémas publics, helpers compilés statiquement sans évaluation dynamique. Appels SSR uniquement par API_WORKER, Request absolue interne ; navigateur uniquement par le proxy `/api/v1/*` de même origine. Aucun accès direct D1/R2. Desktop conserve l’URL HTTPS API existante indépendante.

Le proxy de lecture limite les routes et méthodes, refuse les origines étrangères et les redirections, filtre les en-têtes entrants/sortants, impose no-store privé et ne relaie des cookies de réponse qu’avec HttpOnly/SameSite et Secure en HTTPS. Il ne crée pas de session, de jeton navigateur ou de route d’administration. Le routage OIDC et ses écritures restent aux étapes dédiées. Les délais d’API et la taille des réponses JSON sont bornés ; téléchargement relayé en flux.

Wrangler déclare API_WORKER vers les workers local/staging/production correspondants, sans route domaine ni workers.dev activé. Le build SSR n’est pas un déploiement ; le binding staging désigne l’API déjà déployée, son invocation distante depuis le site n’est pas revendiquée. `/` affiche un état 503 si API absente ; aucune fixture de remplacement. L’atelier et ses données restent hors du build, contrôlés aussi dans les JS/JSON émis. Les déclarations de types Cloudflare sont limitées à la frontière env pour éviter le conflit des types Worker/DOM dans les scripts navigateur.
