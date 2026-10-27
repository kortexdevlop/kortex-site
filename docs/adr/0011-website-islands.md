# ADR-0011 — îlots Preact et site SSR

Date : 2026-10-27. Décision technique dans le lot 10.3, 10.4, 11.0 autorisé par le propriétaire.

Un seul framework d’îlots : Preact, intégré par @astrojs/preact. Les composants statiques restent Astro ; seule la recherche nécessite un état interactif. Aucun React/compat ou second framework introduit. TypeScript strict avec jsxImportSource preact.

Le site utilise l’adaptateur Cloudflare 14.4.2 compatible avec Astro 7.4.2, import `env` depuis `cloudflare:workers` et Service Binding API_WORKER. Aucune DB ni R2 directement accessible au site. Les cookies restent côté HTTP ; le proxy public est volontairement limité aux lectures de catalogue/téléchargement, ne relaie pas Authorization ni les en-têtes d’administration, refuse les redirections et impose no-store. Les routes d’authentification seront conçues dans leur étape propre.

Les fixtures de démonstration sont importées uniquement par la route injectée en dev, hors du routage produit. Le build contrôle leur absence dans les fichiers JS/HTML/JSON émis. La référence manque toujours : pas de gel ni de fidélité visuelle revendiqués, poursuite technique autorisée par « Enchaîne les 3 prochaines étapes » après réserve explicite.
