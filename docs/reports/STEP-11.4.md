# Rapport 11.4 — Référencement dynamique

27/10/2026, Windows. Base 524a6b0. Lot de trois étapes expressément autorisé par le propriétaire. Résultat Git dans PROGRESS.md ; aucune approbation visuelle formelle inventée.

Sitemap-index et segments XML SSR de 500 IDs, robots.txt, OpenGraph et JSON-LD SoftwareApplication sans notes inventées. Canonicals et échappement XML/JSON conservés. published_at est un timestamp en millisecondes, comme le publieur réel. Cache SEO borné à 900 secondes, erreurs no-store ; aucune construction de 10 000 routes Astro et aucune itération des listes publiques par paquets de 100.

Index D1 partiel idx_tools_sitemap, migration 0004 appliquée localement et sur la D1 staging possédée par le projet. L’endpoint interne est exporté exclusivement par la classe WorkerEntrypoint SitemapService et un Service Binding SITEMAP_WORKER séparé. La route publique de l’API et le proxy Web renvoient 404 pour cette URL ; aucun en-tête HTTP usurpable ne donne accès à l’endpoint interne.

Tests : parcours réel des 10 000 IDs D1 en 20 segments sans doublons/trous/non publiés ; XML parsé dans Chromium/WebKit, dates, 404 hors bornes, robots, OpenGraph, JSON-LD et sérialisation hostile. Les outils masqués, /app et comptes privés ne figurent pas dans le sitemap. Staging reste non indexable par X-Robots-Tag, tout en exposant ses endpoints pour validation.

API : 3 tests DTO et 28 tests Worker réussis, typecheck strict réussi. Site : 51 fichiers sans diagnostic, 16 tests Node, build sans atelier/fixtures. Les suites du lot comptent 8 essais locaux et 2 contrôles distants Chromium/WebKit réussis. Preuves consolidées : docs/evidence/website-11.4-11.6-validation.json. Contrats publics existants inchangés ; aucun changement DNS ou messagerie.
