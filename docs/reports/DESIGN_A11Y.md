# Interface et accessibilité : 20.3

Implémentation 1c77871215fd405d02835d55f90c1d4890b355f9. 33 tests Chromium réussis sur accueil/catalogue/fiche outil/compte anonyme/assistant/parcours/bureau aux largeurs 1632/1024/768/390. Axe : zéro violation sérieuse ou critique sur les états audités, sans certification WCAG complète.

Textes secondaires et placeholders plus contrastés ; raccourci sur bouton noir conservé lisible. Nom du logo aligné sur son texte. Dock vide sans rôle tablist invalide, carte des parcours nommée et défilable au clavier. 76 boutons visibles activés atteints par Tab sur six routes, sections repliables ouvertes au clavier. Menu mobile, dialogues, Escape, restauration du focus et lien d’évitement vérifiés. Cinq interfaces d’outils isolées auditées, permission CSV et refus réel sans fichier ni upload. Erreur 404, reduced-motion et reflow au viewport équivalent 200 % vérifiés ; ce dernier contrôle ne manipule pas le zoom natif du navigateur.

Narrateur Windows réellement démarré : lien d’évitement vers main, scan H vers Récents, Shift+E vers champ de besoin, focus cyan observé et arbre UIA conservé. Aucun jugement auditif ni enregistrement de voix ; smoke technique, pas une revue humaine exhaustive. Narrateur et fenêtre de test arrêtés.

Image fournie intacte, SHA256 22b72dd91b43675643c8bf996074690916bec86b5ebd55d785aeff2fae603067. Capture native 2706×1764, sept régions à ±4 px ; onze zones plates, Delta E 2000 maximal 2,03276 <12 ; erreur RGB moyenne 5,168432/255. Aucun pourcentage global de ressemblance ni pixel perfect revendiqué. Textes volontairement foncés pour accessibilité : différence motivée. Police originale inconnue ; pages sans référence non comparées pixel à pixel. Démonstration visuelle séparée du vrai catalogue.

Preuves : docs/evidence/a11y-20.3/ (résultats axe, couverture clavier, géométrie/couleurs, 14 captures desktop/mobile, captures et trace Narrateur). Baseline a11y-baseline-20.3.json conservée. Revue humaine et staging restent des gates distincts.
