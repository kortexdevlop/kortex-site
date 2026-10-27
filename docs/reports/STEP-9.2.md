# Étape 9.2 — API locale prête, arrêt au contrôle Cloudflare

Date : 2026-10-27. Base : 8237f5e. Résultat : commit contenant ce rapport. Prérequis : 9.1 validée. Statut : préparation locale terminée ; étape laissée en attente à la demande du propriétaire, devant les autorisations Cloudflare insuffisantes. 9.3 non démarrée.

api/ contient le Worker TypeScript strict, Env contrôlé par Zod, routes /api/v1, erreurs JSON et liveness exacte `{status:"ok",api_version:"1"}`. HEAD, méthodes, CORS à origine exacte, prérequêtes, UUID de corrélation et headers CSP/no-store/nosniff sont contrôlés. Les configurations/secrets et erreurs internes ne sont jamais renvoyés en HTTP. Les bindings locaux DB/TOOLS_BUCKET sont réels, AI absent. Les environnements distants n'ont ni ID réel ni binding provisionné ; .invalid et absence de bindings font échouer la validation de configuration.

shared/schemas/api.schema.json ajoute les DTO candidats santé, erreurs, catalogue/détail/actions, publication et pagination bornée. Les routes catalogue/publication ne sont pas implémentées par anticipation. Les six schémas gelés restent inchangés. Le workspace listait déjà api ; sa configuration ne change que par l'exception ciblée d'âge du paquet de types, et le lock active le nouveau package.

| Commande | Date / environnement | Sortie | Résultat |
|---|---|---:|---|
| pnpm --filter @kortex/api test | 2026-10-27, Windows, Node 24.15, Vitest 5 / vrai workerd | 0 | 3 tests DTO Node et 6 tests Worker ; santé, erreurs, CORS, secrets, D1 batch rollback et R2 local |
| pnpm --filter @kortex/api typecheck | Windows, TypeScript 7.0.2 | 0 | Sources et tests stricts |
| pnpm --filter @kortex/api build | Windows, Wrangler 4.152.0 | 0 | Build --dry-run explicite, aucun upload |
| node api/scripts/smoke-local.mjs | Windows, vrai serveur wrangler dev --local | 0 | HTTP 200 avec payload exact, 404 JSON ; processus propre arrêté |
| pnpm --filter @kortex/api exec wrangler d1 migrations apply DB --local --persist-to ../target/api-d1-state-final | D1 local | 0 | 0001 et 0002 appliquées via les bindings de l'API |
| pnpm install --frozen-lockfile | Windows/pnpm 11.25.0 | 0 | Lock et dépendances vérifiés |

Premier essai non réussi : le workerd verrouillé accepte au maximum compatibility_date 2026-10-26, et a refusé 2026-10-27 dans Vitest. La configuration API a été fixée à la date supportée, sans mise à jour de moteur. Le type Env de la version actuelle est déclaré via Cloudflare.Env/GlobalProps. Les contrôles finaux passent. @cloudflare/vitest-plugin 1.4.7 remplace le nom historique du pool dans le plan et conserve Wrangler/Miniflare/Vitest de la baseline.

## Contrôle distant et arrêt demandé

La recherche d'intégration Cloudflare ne trouve aucun connecteur disponible. Wrangler whoami --json, exécuté en lecture seule, établit cependant une session OAuth déjà authentifiée et un compte. Ses scopes annoncés couvrent user/account, Pages et zone, sans Workers/D1. D1 list et R2 bucket list sont refusés, code 10000 ; les lectures des déploiements nommés staging/production échouent également. Les ressources ne sont donc **pas vérifiables**, et ne sont pas déclarées absentes. Aucun account_id ni binding D1/R2 distant n'est configuré pour Kortex.

Conformément à l'instruction utilisateur, aucun login, changement de permission, provisionnement ou déploiement n'a été lancé. Le rapport d'identité conservé est expurgé : pas de token, email ou ID de compte. infra/cloudflare-setup.md donne l'état exact, la commande de réautorisation avec scopes sélectionnés et les lectures à reprendre ensuite. La reprise n'est pas un accord de déployer avant 9.6. Preuve : docs/evidence/validation-9.2-local.json et docs/evidence/cloudflare-checkpoint-9.2.json. Linux/macOS et Firefox restent exclus.
