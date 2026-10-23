# Preuves locales — 4.4

Windows, 23 octobre 2026, Dioxus 0.7.10, dépendances verrouillées. Aucun contrat ABI/schéma modifié.

Les sept tests de crates/kortex-ui/tests/render.rs couvrent :
1. Correspondance exacte du registre aux 17 types FROZEN_V1 et snapshot de l'arborescence rendue.
2. Labels accessibles, ARIA tabs/panels, tabulation native et calcul des déplacements clavier.
3. Injection HTML échappée dans texte, titres, boutons, alertes, tables et SVG.
4. Refus avec message visible des types inconnus ; clés dupliquées, profondeur 33, 2 001 nodes, message supérieur à 1 Mio et séries mal dimensionnées.
5. Six niveaux de titres, disabled, graphiques vides/zéro/négatifs/extrêmes, plusieurs instances aux IDs distincts.
6. Vrais handlers Dioxus click/input/change/select via VirtualDom, conformité à $defs/event, booléen checkbox, disabled silencieux, sélection d'onglet par ArrowRight et rendu actualisé.
7. Fallback commun sur propriété interdite pour chacun des 17 types et échappement des labels/titres/textes.

Résultat : 7/7 tests ; cargo check natif et wasm32 réussis ; fmt réussi. Cinq empreintes de schémas gelés inchangées.

Contrôles navigateur reproductibles : tests/browser.cjs lit le HTML de Dioxus généré par le test all_frozen_types_and_snapshot. Chromium et WebKit passent l'ordre de Tab bouton/input/textarea/select/checkbox/summary/onglet/panel, association de tous les labels, unicité des IDs, valeur initiale textarea, audit axe WCAG2A/WCAG2AA/WCAG21AA sans violation et absence de débordement horizontal à 360 px. Résultats machine : ui-4.4-browser.json ; captures revues : ui-4.4-chromium.png et ui-4.4-webkit.png.

Limite de preuve : le navigateur teste ici le rendu SSR et le comportement natif de tabulation. Le clavier de sélection est testé avec les handlers Dioxus dans VirtualDom ; un vrai adaptateur devra encore exercer MountedData::set_focus et la boucle UI/moteur. Les tests actuels ne prétendent pas certifier toute l'accessibilité ni une UI finale complète.
