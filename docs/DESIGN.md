# Fondations visuelles Kortex — 27 octobre 2026

Statut : proposition technique locale ; fidélité à la référence non validée.

## Source et limites

Le plan cite une capture d’accueil du 23/10/2026, 1632 × 927. Son fichier n’est pas dans le dépôt, les pièces jointes disponibles ni les archives Kortex inspectées. Le fichier « Tableau de bord Kortex OS apaisant.png » (1586 × 992) représente un autre concept ; il n’est pas utilisé comme référence. Les anciennes archives contiennent un autre design system, non repris. Aucun écran n’est fabriqué pour remplacer la référence.

Les éléments **décrits par le plan, pas observés sur l’image** : fond blanc cassé, carré K noir, navigation horizontale, hero à deux colonnes, grand titre noir/gris, recherche arrondie, pastilles, neuf cartes pastel 3 × 3 et icônes avec dégradés/ombres. La présence exacte, les coordonnées, les couleurs, la police et les dimensions restent inconnues. Mobile : **à concevoir**. Dark mode, pied de page et réglages : interprétations proposées, pas reproduction attestée.

## Inventaire préparé dans ce lot

Header, logo typographique provisoire K, navigation Outils/Applications/Télécharger/Compte, SearchField, Button principal/secondaire/désactivé, Pill lien ou badge, Shell/MainLayout et Footer. États hover, focus, sélection et désactivation. L’atelier montre ces primitives ; son bloc décoratif ne constitue pas une nouvelle référence. Les cartes de catalogue et leur grille relèvent de 10.3 et ne sont pas implémentées.

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

`pnpm --filter @kortex/website dev` ouvre l’atelier local à `http://127.0.0.1:4321/__dev/design`. Sa source est `src/previews/design.astro`, hors de `pages`, et la route est injectée uniquement pour `astro dev`. Ce déplacement volontaire par rapport au chemin envisagé dans le plan évite toute inclusion en production ; le build vérifie l’absence de route/contenu de développement. Le seul document statique produit dans ce lot est une page 404 de préparation. Aucun déploiement ni route produit finale n’a lieu avant la phase 11.
