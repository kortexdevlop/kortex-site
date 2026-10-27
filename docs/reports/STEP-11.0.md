# Rapport 11.0 — Astro SSR et frontière API

Date : 2026-10-27. Base : d4e9225. Résultat : commit de ce lot référencé dans PROGRESS.md.

Socle SSR techniquement implémenté et testé localement sur instruction du propriétaire de poursuivre le lot. Statut formel pending derrière la réserve visuelle 10.0/10.4 ; pas de déploiement ou de gel visuel revendiqué.

Astro 7.4.2 / adaptateur Cloudflare 14.4.2, Tailwind conservé, TypeScript strict et Preact unique. Wrangler déclare les bindings local/staging/production, sans route domaine ou workers.dev. Import env actuel depuis cloudflare:workers, aucun ancien locals.runtime. API_WORKER est la seule source métier côté SSR ; client listTools/getTool/getActions validé sur les schémas existants. Aucun accès D1/R2 depuis le site. Desktop garde sa propre URL HTTPS API, sans modification.

Proxy de lecture `/api/v1/*` : liste fermée routes/méthodes, contrôles d’origine, absence de relais Authorization/en-têtes admin, refus des redirections, no-store privé et cookies HttpOnly/SameSite avec Secure en HTTPS. Aucun mécanisme de connexion simulé, aucune écriture ni publication relayée. Timeout des appels et limite JSON 1 Mio contrôlés, téléchargement en flux. Canonicals protégés contre les URL hors origine.

`/` appelle réellement le catalogue API, renvoie 503 et un message explicite si indisponible ; aucune fixture de remplacement. État 200 de préparation si API valide. Favicon hôte provisoire dérivé du K, référence visuelle toujours non confirmée. Atelier injecté seulement en dev, contrôle d’absence de route et fixtures dans le JS/HTML/JSON produit.

| Commande | Date / environnement | Résultat |
|---|---|---|
| pnpm --filter @kortex/website check | 2026-10-27, Windows | 0 diagnostic strict ; 10 tests réussis |
| pnpm --filter @kortex/website build | 2026-10-27, Windows | build SSR, manifeste Worker/favicon présents, atelier/fixtures absents |
| pnpm exec wrangler dev --config dist/server/wrangler.json --local --ip 127.0.0.1 --port 4330 --var WEB_ORIGIN:http://127.0.0.1:4330 | 2026-10-27, website, workerd local | Worker compilé lancé, API non connectée |
| node website/scripts/smoke-ssr.mjs http://127.0.0.1:4330 | 2026-10-27, Windows | 8 contrôles HTTP réussis : accueil 503, dev 404, favicon/CSS 200, API 503, POST 405, origine étrangère 403, admin 404 |
| pnpm --filter @kortex/api test | 2026-10-27, Windows workerd | 3 DTO + 26 Worker réussis après résolution finale des dépendances |

Logs, rapport HTTP structuré et empreintes : docs/evidence/validation-10.3-11.0.json. Erreurs initiales et corrections : STEP-10.3.md. Installation frozen et absence de secrets contrôlées dans la validation consolidée. Sessions Cloudflare : binding KV local généré par l’adaptateur, aucune ressource distante créée. Aucun test distant du site ou raccordement effectif au binding staging revendiqué ; seul l’API staging existante est déjà déployée. Linux/macOS et Firefox exclus sur instruction du propriétaire. Domaine et messagerie conservés.
