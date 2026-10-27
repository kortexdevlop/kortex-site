# Rapport 10.4 — démonstration et baseline candidate

> Actualisation : une nouvelle référence utilisateur 2706×1764 a été reçue puis reproduite le 27/10/2026. Les mentions de capture manquante et du concept initial ci-dessous sont historiques. État courant : docs/reports/DESIGN-WORKSPACE-2026-10-27.md et docs/evidence/workspace-validation.json. Aucune approbation de fidélité après résultat n’est inventée.

Date : 2026-10-27. Base : d4e9225. Résultat : commit de ce lot référencé dans PROGRESS.md.

Implémentation de démonstration et tests réalisés dans le lot autorisé ; **validation visuelle finale non satisfaite**, original toujours absent. Statut formel pending derrière 10.0. Pas de gel ou de revue humaine inventés.

Neuf ToolSummary de démonstration validés, importés uniquement par l’atelier `/__dev/design`, sans catalogue public. Hero et primitives existantes conservés ; section des cartes, recherche et états ajoutés. Captures viewport exactes 1632×927, 768×1024 et 390×844, plus captures de la section de neuf cartes ; DPR 1, polices système Windows et mouvement réduit. Colonnes calculées 3/2/1, absence de débordement, Tab et axe contrôlés sur Chromium/WebKit. Le header conserve ses essais clair/sombre, mobile, stockage et langue.

Commandes : `pnpm exec playwright test tests/design/cards.spec.ts tests/visual/design-system.spec.ts tests/visual/header.spec.ts`, `pnpm --filter @kortex/website check`, `pnpm --filter @kortex/website build`. Date : 2026-10-27, Windows. Sorties, nombre de tests, rapports JSON et SHA-256 dans docs/evidence/validation-10.3-11.0.json ; captures library-* et navigation-*.

docs/DESIGN.md et docs/evidence/design-baseline.md décrivent les arbitrages responsive comme **conçus**, sans les attribuer à une image absente. Aucune coordonnée, couleur échantillonnée, comparaison pixel ou CIEDE2000 à l’original n’est inventée. Seuils proposés 10.0 non appliqués, validation header/hero/cartes en attente de récupération de la capture et de revue.

Pas de hacks CSS propres à une page ; grilles partagées, points de rupture uniques dans les tokens. Pas de mock permanent ni de donnée demo dans le build. Domaine et messagerie inchangés ; 11.1 non engagée.
