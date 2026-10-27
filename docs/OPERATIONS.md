# Exploitation du site Kortex

## Environnement de validation

Le site et le Runtime de staging utilisent exclusivement les Workers kortex-web-staging et kortex-run-staging sur workers.dev. L’API, D1 et R2 restent les ressources de staging déjà possédées par le projet. Aucun DNS, routage de messagerie ou service de production n’est modifié. Le site de staging porte X-Robots-Tag: noindex, nofollow.

Déploiement reproductible : construire le site, exécuter ses contrôles locaux puis `node scripts/web-hot-publish.mjs`. Ce script vérifie le compte et l’identité des ressources, déploie l’API/site/Runtime de staging, publie un fixture signé unique, l’exécute réellement dans Chromium et WebKit, le révoque et compare l’identifiant du déploiement Web ainsi que le hash du build avant/après. L’opérateur de publication est temporaire, sa clé privée reste en mémoire et la politique antérieure est restaurée à la fin. Il s’agit d’un opérateur local simulant le pipeline CI, sans revendication d’un passage GitHub OIDC.

Les configurations générées sont sous target/web-hot-publish ; elles ne remplacent pas la configuration de production. Pour tester ensuite l’état distant : `pnpm exec playwright test --config tests/integration/playwright.hot.config.ts`. Le test refuse un rapport de publication manquant ou non réussi.

## Cache, nouvelle publication et révocation

- Catalogue et fiches publiques : Cache-Control public, max-age=60. Recherche via proxy API privé no-store.
- Sitemap/index/robots : max-age=900 ; les dates correspondent au published_at réel en millisecondes.
- App, frame et configuration du Runtime : no-store. Le Runtime relit la fiche publiée via Service Binding avant d’admettre un lancement et avant de livrer le descripteur signé.
- Téléchargement Runtime : proxy de l’API, sans cache HTTP. Le cache local d’archives du loader est borné, vérifié à nouveau et ne dispense jamais de l’admission API d’un nouveau lancement.
- Une révocation retire la fiche/actions/sitemap et refuse les nouveaux lancements/téléchargements ; elle n’efface pas les octets déjà détenus par une personne ni ne prétend interrompre un calcul hors ligne déjà commencé.

Aucune mise en cache HTML supplémentaire dans caches.default ou règle CDN n’est activée ici. Si une telle couche est ajoutée, conserver les TTL maximaux, purger les URLs de catalogue/fiche/sitemap concernées et refaire le gate de révocation avant activation. Les artefacts R2 sont immuables ; retirer un outil par révocation et conserver l’audit, sans écrasement du paquet.

## Mesure de recherche

`node scripts/benchmark-search-staging.mjs` crée un Worker et une D1 temporaires avec 10 000 enregistrements synthétiques distincts du catalogue du produit. Il mesure 25 aller-retours HTTPS API avec lecture/validation des résultats, calcule le p95, puis supprime uniquement ces ressources nouvellement créées. Le délai de saisie de 250 ms est une valeur UI séparée de cette mesure réseau/API. Rapport : docs/evidence/search-staging-benchmark.json.
