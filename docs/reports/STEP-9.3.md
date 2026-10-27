# Rapport d'étape 9.3

Statut : implémentée et testée localement, awaiting_approval pour la revue du résultat. Date : 2026-10-27. Base : 088bceec414e0dcd7f4282402604abff549cc278. Résultat : commit 0a21749. L'instruction du propriétaire autorise la réalisation de l'étape après nettoyage ; aucune revue indépendante ni approbation après résultat n'est inventée.

Prérequis : 9.2 validée localement, profil kortex actif. Retrait distant de l'ancien projet effectué séparément et documenté dans CLOUDFLARE-RESET-2026-10-27.md ; le propriétaire a demandé de conserver la messagerie. Aucune ressource du nouveau projet provisionnée.

L'API partage maintenant liste, recherche, détail et actions de la version publiée courante. Les deux ordres utilisent leurs clés composites stables et les index existants. Tous les filtres sont préparés ; q, category, tag, sort, limit et cursor sont bornés et contrôlés, les répétitions et inconnus refusés. Le curseur strict HMAC-SHA256 lie les filtres, tri et limite à la position ; la clé serveur dédiée n'a pas de valeur applicative par défaut. Les erreurs internes et secrets ne sont jamais renvoyés. Les DTO de sortie sont validés avec du code généré depuis les schémas, et les identités croisées sont vérifiées. Les actions proviennent du contrat immuable courant, jamais de la version draft suivante.

ETag SHA-256, If-None-Match/304, HEAD, méthodes/Allow et CORS exact sont contrôlés. Le statut de publication est lu avant le calcul conditionnel : l'ancienne représentation d'un outil dépublié ne produit pas de 304. Les incohérences de métadonnées du détail et des résumés sont refusées. Les erreurs restent no-store, les réponses publiques exigent une revalidation.

| Commande | Environnement / date | Sortie | Résultat observé |
|---|---|---:|---|
| pnpm --filter @kortex/api test | Windows, 2026-10-27, Vitest 5.0.5, vrai D1/workerd | 0 | 3 tests DTO Node + 15 tests Worker, dont les 2 parcours complets de 10 000 outils |
| pnpm --filter @kortex/api typecheck | TypeScript 7.0.2 | 0 | Sources et tests stricts |
| pnpm --filter @kortex/api build | Wrangler 4.152.0, dry-run | 0 | Bundle 964,83 Kio, aucune livraison |
| node api/scripts/smoke-local.mjs | Vrai serveur wrangler dev --local, Windows | 0 | Santé 200, inconnue 404, arrêt du seul serveur lancé |
| node --test tests/publication/golden.test.mjs | Node 24.15.0, Windows | 0 | Six schémas gelés et référence PACKAGE_FORMAT_V1 inchangés |
| node --test tests/config/secrets.test.mjs | Fichiers ajoutés à l'index Git | 0 | 6 tests, dont tous les fichiers ajoutés à l’index Git |

Les deux parcours recent/name couvrent chacun 100 pages de 100 éléments, comparent l'ensemble exact des 10 000 identifiants et l'ordre de chaque clé, y compris les ex æquo. Les autres tests contrôlent filtres combinés, recherche littérale sans joker/injection, champs invalides/dupliqués/inconnus, curseurs altérés, mauvaise clé/type/scope, version courante avec draft plus récent, outils absents/draft/révoqués, cache/CORS/HEAD/405, clés manquantes et erreurs DB sans fuite, retrait du catalogue et corruption de métadonnées. Les migrations 0001 et 0002 sont réellement appliquées ; les attestations d'état artificielles du jeu de données sont expressément TEST et ne prouvent pas une signature/R2 réelle. Aucun mock permanent ni donnée de démonstration dans le produit.

Échecs intermédiaires corrigés : types du module cloudflare:test et option TextDecoder adaptés aux dépendances installées ; le test supplémentaire de corruption recherchait initialement un nom modifié sans modifier name_norm, donc ne sélectionnait pas la ligne. Le filtre de ce test a été corrigé pour sélectionner réellement la ligne par sa clé normalisée existante. Le dernier passage intégral réussit. Les empreintes des logs finaux sont dans docs/evidence/validation-9.3-local.json ; fichiers de log locaux sous target/.

Limites : stabilité prouvée sur dataset figé, pas de snapshot interrequêtes d'un catalogue en mutation ; recherche sans moteur linguistique (lower du résumé selon SQLite/ASCII) ; rotation de clé invalidant les curseurs existants ; clé réelle à fournir séparément via api/.dev.vars ignoré ou secret d'environnement au provisionnement. Les routes de téléchargement, publication et authentification ne sont pas anticipées. Aucun changement des six schémas gelés ni du DTO API candidat. Linux/macOS et Firefox restent exclus. Prochaine étape prévue : 9.4, non engagée.
