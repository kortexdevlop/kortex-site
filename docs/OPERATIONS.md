# Exploitation de Kortex — staging Windows/Chromium

Les déploiements réels passent par les workflows protégés GitHub, avec qualité vérifiée sur le même commit et identité OIDC de main. Production, DNS et messagerie restent hors des changements de ce lot. Le Runtime garde son origine distincte. Le catalogue réel ne reçoit aucune des 10 000 fiches du benchmark.

## Publication et reprise

Le publisher `scripts/ci-publish-tool.mjs` utilise le moteur commun `scripts/release-core.mjs` : sauvegarde metadata et bookmark D1, upload adressé par hash, publication atomique, audit et réconciliation. Les commandes opérateur `promote-release.mjs`, `rollback-release.mjs`, `revoke-artifact.mjs` exigent un plan explicite. Garder le même request_id et le même contenu pour reprendre une coupure ; ne jamais éditer une opération déjà engagée. Voir `docs/reports/RELEASE_PIPELINE_GATE.md` et `infra/releases/README.md`.

Le workflow tools expose la maintenance staging uniquement sur déclenchement explicite. Le document de révocation doit être signé hors CI et conserver tous les tombstones. Ne pas restaurer une D1 pour annuler un simple déploiement Worker : le rollback de code conserve le schéma, et une migration incompatible demande un correctif compensatoire. Le rollback Desktop annonce un canal signé ; il ne force pas l’installation.

## Audit et sauvegarde

Les opérations D1 sont append-only par triggers. Le journal local vérifie une chaîne de hashes et les copies R2 utilisent des clés d’événement uniques. Time Travel est vérifié par bookmark avant opération ; aucun restore destructif automatique. Cette immutabilité applicative n’est pas un verrou WORM contre l’administrateur Cloudflare. La durée de rétention/locks R2 et le drill incident complet restent à arrêter en phase 21.

## Cache et retrait

Les listes et détails API portent un ETag et `max-age=0, must-revalidate`. L’HTML catalogue/fiche peut garder son TTL court configuré ; sitemap et robots ont un maximum de 900 secondes. Aucun caches.default supplémentaire ni nouvelle règle CDN. Le retrait refuse les nouveaux téléchargements et lancements, sans prétendre effacer les copies déjà détenues ni interrompre un calcul hors ligne commencé.

## Charge isolée

Après un build Astro valide, `node scripts/qa-isolated.mjs` crée des ressources privées temporaires distinctes, vérifie les coupures de release, importe les fiches synthétiques par lots, mesure deux séries de 100 requêtes à 10 req/s, contrôle pagination, SSR et sitemaps, ajoute la 10001e fiche sans rebuild puis nettoie ses seules ressources. Les secrets restent en mémoire et en bindings secrets ; expiration d’accès au bout de deux heures. Un échec de nettoyage est un échec du gate et reste inscrit dans la preuve datée.

Ne pas utiliser les anciens scripts `web-hot-publish.mjs` ou `benchmark-search-staging.mjs` pour ce gate : ils décrivent des campagnes historiques et certains accèdent au bucket partagé ou à des navigateurs désormais exclus. Référence actuelle : `docs/QA_PLAN.md`, `docs/reports/QA_BASELINE.md` et `docs/reports/CATALOG_SCALE_10K.md`.
