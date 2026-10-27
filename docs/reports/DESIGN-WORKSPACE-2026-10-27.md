# Nouvelle référence utilisateur — espace de travail

Demande : « Voici le design à suivre de maniere ultra stricte et identique au maxium ». Cette instruction explicite remplace la proposition visuelle antérieure ; elle ne modifie aucun contrat de tool/runtime/API. Date : 2026-10-27. Base : b7ffda5. Résultat : commit contenant ce rapport, identifié dans PROGRESS.md.

Source reçue : `C:/Users/µ/Desktop/Kortex Workspace-selection.png`, 2706×1764, SHA-256 `22b72dd91b43675643c8bf996074690916bec86b5ebd55d785aeff2fae603067`. Copie exacte `docs/reference/workspace-selection.png`, jamais modifiée et jamais utilisée comme fond de l’interface. Il s’agit d’une **nouvelle référence**, sans présumer qu’elle soit l’ancienne capture 1632×927 du plan.

Implémentation : WorkspaceLayout / Workspace réutilisables pour l’aperçu et la racine SSR ; source de styles mesurés et tokens communs, police du titre auto-hébergée avec licence ; sidebar, nouvelle tâche, catégories/compteurs, état Desktop, compte, en-tête/recherche/Docs, demande texte et fichier local, suggestions, récents et grille de quatre cartes blanches. Les structures antérieures restent seulement dans `/__dev/components`. La grille trois colonnes/pastels du premier concept n’est plus la direction de l’interface publique.

Les fixtures des 14 outils, trois récents, compte KS et statut Desktop de la capture sont uniquement dans l’aperçu dev. La racine utilise les réponses API validées, ne fabrique ni historique ni connexion Desktop ; erreur 503 visible si API absente. Les modes, filtres, favoris locaux, grille/liste, raccourcis et dialogues fonctionnent. Le fichier déposé demeure local, sans traitement ou envoi simulé.

Contrôles : `pnpm --filter @kortex/website check`, `pnpm --filter @kortex/website build`, `pnpm exec playwright test tests/visual/workspace.spec.ts`, suite de compatibilité des primitives conservées, comparateur lecture seule `scripts/compare-workspace.py`, contrôle HTTP du Worker compilé et scan de secrets. Codes, comptes, logs et empreintes sont consignés dans `docs/evidence/workspace-validation.json`.

Comparaison : géométrie des sept blocs principaux dans une tolérance de 4 pixels source, captures aux dimensions natives, échantillons CIEDE2000 contrôlés ; MAE RGB descriptive sur la totalité du raster. Les résultats ne constituent pas un pourcentage de similarité perceptuelle ni une identité pixel parfaite. Typographie/anticrénelage restent les écarts principaux. Petit écran conçu séparément (aucune référence mobile fournie).

Accessibilité : sémantique, commandes clavier, focus, menu mobile, dialogues et absence de débordement testés. Les gris faibles sont repris pour respecter la consigne de fidélité ; le rapport axe conserve les violations de contraste plutôt que de les masquer. Aucune conformité WCAG AA globale du nouvel espace n’est revendiquée. Les contrôles de contraste du premier jeu de tokens restent valables uniquement pour ce premier jeu.

Premiers échecs corrigés : script TS global en conflit avec window.status, devenu module explicite ; assertion de test cliquant directement un radio visuellement masqué, remplacée par son interaction clavier native. Aucun succès inventé. Aucun déploiement ni changement de DNS, messagerie ou service distant. Pas d’approbation humaine après résultat enregistrée automatiquement.
