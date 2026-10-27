# Rapport 10.3 — cartes, recherche et états

Date : 2026-10-27. Base : d4e9225. Résultat : commit de ce lot référencé dans PROGRESS.md.

Autorisation : « Enchaîne les 3 prochaines étapes » après signalement explicite de la capture manquante. Implémentation technique réalisée ; statut formel pending derrière la validation visuelle 10.0. Aucune dérogation ou approbation de fidélité inventée.

ToolCard(ToolSummary), ToolDetails(ToolDetail), ToolGrid, Feedback et Search.tsx livrés. Schémas publics existants utilisés pour valider les entrées ; palette stable par catégorie, texte échappé, titre tronqué à deux lignes mais complet dans le DOM. Icône de repli hôte par monogramme ; aucun asset d’outil non vérifié chargé. L’affichage des assets de paquet attend une source validée, sans détour par un URL icon_path arbitraire.

Un seul framework d’îlots Preact, documenté ADR-0011. Recherche explicite accessible, réponses validées, résultats obsolètes ignorés, erreur/vide/chargement. Les exemples sont limités à la route dev et exclus du bundle produit. Aucun outil fictif sur le site public, aucune route individuelle créée.

| Contrôle | Date / environnement | Résultat |
|---|---|---|
| pnpm exec playwright test tests/design/cards.spec.ts tests/visual/design-system.spec.ts tests/visual/header.spec.ts | 2026-10-27, Windows Chromium/WebKit | Rapport JSON consolidé : cartes, 3/2/1 colonnes, clavier, filtres, échappement, états et accessibilité |
| pnpm --filter @kortex/website check | 2026-10-27, Windows | Typecheck strict et 10 tests réussis, dont source unique/contrastes, DTO, proxy et délais |
| pnpm --filter @kortex/website build | 2026-10-27, Windows | SSR compilé et fixtures absentes |

Preuves et empreintes : docs/evidence/validation-10.3-11.0.json. Échecs initiaux corrigés : déclarations Worker globales en conflit avec DOM ; helpers CommonJS Ajv non utilisables directement dans workerd dev, désormais bundlés en ESM autonome ; test historique ciblant tous les scripts main au lieu de la seule zone texte ajusté pour distinguer l’hydratation légitime de l’injection HTML. Le premier chargement après réoptimisation Preact pouvait échouer (__H) lors de la découverte tardive de preact/devtools : correction par optimisation préalable explicite et déduplication de preact. Aucun changement de version destiné à masquer le défaut ; démarrage et suite finale revérifiés.

Réserve : original manquant, fidélité non mesurée, aucun gel de design. Aucun contrat gelé modifié, aucune approbation après résultat.
