# Design Kortex — référence active du 27 octobre 2026

La consigne utilisateur « design à suivre de manière ultra stricte et identique au maximum » fait de **Kortex Workspace-selection.png** la référence active. Elle remplace le premier concept (nav horizontale, K noir et cartes pastel). L’ancien fichier 1632×927 n’est plus un prérequis de cette reproduction ; aucune identité entre les deux captures n’est présumée.

## Provenance

Original : C:/Users/µ/Desktop/Kortex Workspace-selection.png. Copie exacte versionnée : `docs/reference/workspace-selection.png`. Dimensions natives **2706×1764**, SHA-256 **22b72dd91b43675643c8bf996074690916bec86b5ebd55d785aeff2fae603067**. Le raster n’est ni retouché ni utilisé comme arrière-plan de l’interface. Son DPR d’origine n’est pas présumé. Les comparaisons se font en pixels natifs, capture de navigateur au même viewport avec DPR 1.

## Inventaire observé

Deux panneaux arrondis, marge extérieure et gouttière ; sidebar fixe blanc/gris avec marque carrée en dégradé sans lettre, kortex.site et BETA ; bouton Nouvelle tâche, quatre entrées de navigation avec compteurs, six catégories colorées, état de l’app locale et compte KS. Le panneau principal contient un en-tête avec fil Espace de travail / Accueil, recherche globale et Docs, puis un contenu défilant.

Le titre centré est « Que voulez-vous faire aujourd'hui ? », suivi du sous-titre, d’une grande zone de demande blanche, dépôt de fichier, choix Tous / En ligne / Local et bouton d’envoi noir. Quatre suggestions, trois récents, titre Tous les outils / 14 outils, Grille / Liste et cartes blanches en **quatre colonnes**. Les pastels sont limités aux icônes ; pas de grandes cartes pastel dans cette référence. Aucun footer ni commande densité/thème/langue n’est visible : ils ne sont pas ajoutés au nouvel espace.

La première rangée contient Compresseur d’images, Suppression de fond, Redimensionner, Convertisseur PDF. La seconde rangée n’est visible que partiellement : ses icônes/labels supérieurs sont observés, les éléments situés sous le cadrage restent non spécifiés. L’image ne démontre pas qu’une app locale soit réellement connectée ni que les outils existent dans l’API.

## Mesures natives

| Zone | x / y | largeur / hauteur |
|---|---|---|
| Sidebar | 26 / 26 | 444 / 1712 |
| Panneau principal | 498 / 26 | 2182 / 1712 |
| En-tête | 498 / 26 | 2182 / 126 |
| Zone de demande | 815 / 458 | 1520 / 280 |
| Premier récent | 578 / 1008 | 479 / 132 |
| Première carte | 578 / 1320 | 479 / 304 |
| État Desktop | 50 / 1414 | 396 / 176 |

Le comparateur DOM impose ±4 pixels sur ces grandes boîtes, plus strict que les ±16 initialement proposés. Les formes arrondies et contours anticrénelés empêchent d’assimiler une boîte DOM à chaque pixel de bord. Les espaces et dimensions sont proportionnels à la largeur du viewport sur ordinateur : à 1353 px, les mesures sont à demi-échelle. Réglages actifs dans `tokens.json` et `workspace.source.css`.

Couleurs échantillonnées dans l’original : extérieur #eef0f3, sidebar #f8f9fb, fond cyan #d8f4ff, fond lilas #f8eefe, navigation sélectionnée #e7e8ea, encre #111214, icône image #d4f0ff, document #ffe4da, développeur #daf3e1. Les transparences/dégradés reproduisent les autres surfaces. Les points exacts et valeurs comparées sont dans `workspace-comparison.json` ; CIEDE2000 ≤12 proposé, effectivement vérifié uniquement sur ces zones listées.

## Typographie et limites

La police du raster n’est pas identifiée. Texte système Arial/Helvetica ; titre Roboto variable auto-hébergé et réglé optiquement après comparaison, ainsi que certains textes de poids moyen. Licence OFL jointe. Pas de dépendance réseau de police à l’exécution. L’anticrénelage et de légères différences de glyphes/poids restent visibles : aucune identité pixel parfaite ni police originale retrouvée n’est revendiquée.

La MAE RGB et la RMSE sont des mesures descriptives sur le raster entier, dominées en partie par les grandes surfaces claires. Elles ne sont **pas un pourcentage de fidélité perceptuelle**. Les captures de test ne deviennent pas un nouvel original. La revue visuelle après résultat appartient au propriétaire.

## Implémentation et comportement

`WorkspaceLayout.astro`, `Workspace.astro`, `workspace.ts` et les tokens mesurés sont le socle actif pour l’accueil SSR et l’aperçu. Pour les prochaines pages, réutiliser cet espace, pas le Header horizontal ou les grandes cartes pastel du premier concept. L’ancienne bibliothèque reste accessible seulement via `/__dev/components` pour conserver ses contrôles de compatibilité.

Source commune `tokens.json` → `build-design-tokens.mjs` → `theme.css`. Le même générateur transforme l’unité de mesure documentaire wu de `workspace.source.css` en CSS standard dans `workspace.css` (un wu représente un pixel du raster à largeur 2706). Aucun unité wu non résolue ne doit parvenir au navigateur. Le générateur possède un mode check pour les deux exports.

L’aperçu `/__dev/design` importe `tests/visual/workspace-fixtures.json` uniquement en dev : 14 exemples, les trois noms de fichiers, compte KS et statut Desktop de la capture. **La page réelle ne reprend pas ces données comme catalogue ou état de connexion.** Elle consomme les ToolSummary validés de l’API, affiche des compteurs réels de la page reçue, et un 503 explicite sans outils fabriqués si API absente. Aucun historique de fichiers n’est inventé.

Recherche et filtres locaux, catégories, modes, favoris mémorisés sur cet appareil, Grille/Liste, raccourcis, menu et dialogues fonctionnent. Les fichiers choisis/déposés restent locaux ; aucune conversion, IA, ouverture d’outil ou connexion Desktop n’est simulée. L’ouverture complète des outils et le catalogue détaillé restent dans leur étape dédiée.

## Petit écran et accessibilité

Aucune référence mobile fournie : adaptations **conçues** sous 1100 px et 620 px, tiroir de navigation, contrôles réorganisés, grille 2 puis 1 colonne. Les essais couvrent 768×1024, 390×844 et 320×740 sans débordement horizontal. Les modes restent des radios natifs ; focus visible, Échap, retour du focus et dialogue natif. Le dépôt de fichier ne déclenche aucun envoi.

Les gris faibles de l’image sont conservés pour respecter la consigne de reproduction stricte. Les rapports axe du nouvel espace exposent les écarts de contraste ; aucune conformité AA globale n’est déclarée. Les autres violations WCAG A/AA sont contrôlées séparément. Les anciens tests de contraste des tokens initiaux ne sont pas présentés comme une validation des nouveaux gris.

## Vérification et suite

Captures `workspace-chromium-reference.png` / `workspace-webkit-reference.png`, mesures et comparaison dans `docs/evidence/workspace-*`. Original et résultats sont reliés par leurs SHA-256. Le build SSR vérifie l’absence des routes dev et des fixtures, y compris identifiants reference-workspace et noms de récents. Domaine, DNS et messagerie inchangés ; aucune publication dans ce travail.
