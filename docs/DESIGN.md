# Fondations visuelles Kortex — 27 octobre 2026

Statut : proposition technique locale ; fidélité à la référence non validée.

## Source et limites

Le plan cite une capture d’accueil du 23/10/2026, 1632 × 927. Son fichier n’est pas dans le dépôt, les pièces jointes disponibles ni les archives Kortex inspectées. Le fichier « Tableau de bord Kortex OS apaisant.png » (1586 × 992) représente un autre concept ; il n’est pas utilisé comme référence. Les anciennes archives contiennent un autre design system, non repris. Aucun écran n’est fabriqué pour remplacer la référence.

Les éléments **décrits par le plan, pas observés sur l’image** : fond blanc cassé, carré K noir, navigation horizontale, hero à deux colonnes, grand titre noir/gris, recherche arrondie, pastilles, neuf cartes pastel 3 × 3 et icônes avec dégradés/ombres. La présence exacte, les coordonnées, les couleurs, la police et les dimensions restent inconnues. Mobile : **à concevoir**. Dark mode, pied de page et réglages : interprétations proposées, pas reproduction attestée.

## Inventaire préparé dans ce lot

Header, logo typographique provisoire K, navigation Outils/Applications/Télécharger/Compte, SearchField, Button principal/secondaire/désactivé, Pill lien ou badge, Shell/MainLayout et Footer. États hover, focus, sélection et désactivation. L’atelier montre ces primitives ; son bloc décoratif ne constitue pas une nouvelle référence. Le lot 10.3–11.0 ajoute les cartes, les détails validés, leur grille et la recherche ; voir ci-dessous.

Les préférences sont locales, gratuites : densité simple/medium/pro (libellés Simple/Moyenne/Compacte), thème clair/sombre, texte standard/agrandi et FR/EN. « pro » signifie densité compacte et n’accorde aucun droit. Le français demeure sur le contenu de démonstration explicitement balisé comme tel ; seules les chaînes du shell contrôlées sont traduites. Aucun catalogue, compte ou téléchargement n’est simulé. La navigation de l’atelier pointe sur des repères expliquant les futures destinations ; le Header réutilisable prépare les URL /outils, /applications, /telecharger et /compte.

## Source de vérité et accessibilité

`website/src/styles/tokens.json` est l’unique source des thèmes, pastels, espacements de 4 px, rayons, ombres, typographie système, points de rupture et densités. `scripts/build-design-tokens.mjs` génère `theme.css`, versionné et contrôlé par test. Tailwind 4 est intégré par son plugin Vite ; ses couleurs sémantiques pointent sur les variables partagées. Aucune police distante n’est requise. Tous ces choix sont **candidats, non mesurés**.

Textes courants ≥ 4,5:1 sur chaque surface/pastel clair et sombre ; grands textes ≥ 3:1 (le contrôle impose ici aussi 4,5 aux couleurs de texte communes). Focus et contours des champs ≥ 3:1 sur les surfaces principales. Cibles ≥ 44 px. Lien d’évitement, formulaires/légendes natifs, ordre DOM cohérent et menu mobile à divulgation (Entrée/Espace, Échap, retour de focus). Pas de piège de focus pour ce panneau non modal ; son contenu suit le bouton dans l’ordre de tabulation. Sans JavaScript la navigation reste visible. Un stockage indisponible ne bloque pas les réglages pendant la session. Aucun mouvement imposé.

## Protocole de comparaison restant

1. Retrouver le fichier original, consigner son chemin/provenance et SHA-256 ; vérifier 1632 × 927, puis l’annoter sans retouche.
2. Relever les composants réellement visibles, leurs boîtes/alignements, ratios et échantillons couleur ; distinguer les zones absentes ou non spécifiées.
3. Comparer au même viewport, avec tolérances **proposées** positions ±16 px, tailles ±8 %, CIEDE2000 ≤12 par zone ; confirmer ces seuils lors de la revue.
4. Ajuster les tokens et le Header puis refaire les tests et la revue. Les captures générées par nos tests sont des preuves d’exécution, jamais un étalon validé.

La comparaison et la revue de capture annotée sont donc **non exécutées**. Les tests navigateur contrôlent le fonctionnement et l’accessibilité de la proposition, pas sa fidélité. Aucun résultat pixel ou Delta E n’est revendiqué.

## Prévisualisation et compilation

`pnpm --filter @kortex/website dev` ouvre l’atelier local à `http://127.0.0.1:4321/__dev/design`. Sa source est `src/previews/design.astro`, hors de `pages`, et la route est injectée uniquement pour `astro dev`. Ce déplacement volontaire par rapport au chemin envisagé dans le plan évite toute inclusion en production ; le build vérifie l’absence de route/contenu de développement. Depuis 11.0, le site compile en SSR Cloudflare : `/` est un état de préparation réel (503 si API indisponible), sans catalogue inventé ; `/api/v1/*` passe par le binding interne autorisé. Aucun déploiement n’est effectué dans ce lot.

## Bibliothèque candidate — 10.3 / 10.4

ToolCard reçoit exclusivement un ToolSummary validé par les schémas existants. Palette pastel stable par catégorie, titre limité visuellement à deux lignes avec texte complet dans le DOM, monogramme hôte comme icône de repli. Aucun icon_path distant ou SVG d’archive non vérifié n’est rendu : le chargement d’un asset d’outil exige une source de paquet validée qui n’est pas encore raccordée au site. ToolDetails accepte seulement un ToolDetail validé et échappe ses textes. ToolGrid et Feedback centralisent grille, chargement, vide et erreur ; les rendus Astro et Preact partagent modèles, validation, palette, liens et styles.

La recherche est le seul îlot Preact (ADR-0011). Elle soumet explicitement le filtre ; une réponse ancienne ne remplace pas une requête plus récente. Hors démonstration, elle appelle exclusivement `/api/v1/tools` de même origine, avec validation de la réponse et erreurs visibles. Les neuf exemples marqués DEMO sont dans `tests/visual/design-fixtures.json`, uniquement importés par l’atelier. Aucune fixture dans le catalogue public ni dans le bundle de production.

Responsive **conçu**, non mesuré : 3 colonnes à 1632, 2 à 768, 1 à 390 ; seuils uniques dans les tokens (1200 / 768). Cibles de contrôle inchangées, ordre DOM conservé, textes longs cassables et titre accessible. Captures aux viewports prescrits, DPR 1 et polices système Windows, plus captures de la section des neuf cartes. Le thème sombre et les densités restent des interprétations.

Les tokens sont stabilisés dans Git pour cette proposition, **sans gel visuel approuvé**. Aucun écart chiffré à l’original ni Delta E n’est calculé faute de fichier. Voir `docs/evidence/design-baseline.md`. Cette réserve n’empêche pas la préparation technique SSR explicitement demandée après le rapport du lot précédent.
