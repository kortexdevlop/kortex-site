# Baseline visuelle candidate — 27 octobre 2026

Capture d’origine 1632 × 927 toujours indisponible. Comparaison header/hero/cartes, positions ±16 px, tailles ±8 % et CIEDE2000 ≤12 : **non exécutée**. Aucune mesure d’original ni revue humaine de fidélité revendiquée. Les captures de ce lot sont des preuves de rendu de notre proposition, jamais une nouvelle référence substituée.

Les tests `tests/visual/design-system.spec.ts` couvrent Chromium et WebKit Windows aux viewports 1632 × 927, 768 × 1024 et 390 × 844, DPR 1, reduced motion, polices système chargées. Chaque format produit une capture viewport exacte et une capture complémentaire des neuf cartes. `tests/design/cards.spec.ts` contrôle les colonnes calculées 3/2/1, le débordement horizontal, le titre tronqué, l’échappement des textes, le clavier et les états. Axe vérifie WCAG A/AA sur les trois tailles ; les tests du header conservent les contrôles du sombre, du clavier et des préférences.

Arbitrages conçus pour tablette/mobile : navigation repliée sous 1200, grille à deux colonnes jusqu’à 768, puis une ; contenu ordonné sans carrousel, cibles ≥44 px, catégories/textes cassables, titre complet accessible malgré la troncature visuelle. Aucun de ces choix n’est attribué à la capture absente.

Monogrammes hôtes de repli : aucun asset de paquet non vérifié n’est chargé. Les neuf exemples ne constituent pas des outils disponibles ; liens de détail préparés, routes de catalogue final prévues en 11.1. La page publique ne reçoit aucune de ces données de démonstration.

État : bibliothèque testable et socle SSR préparés ; gel de design et validation de fidélité restent en attente. Preuves d’exécution, versions et SHA-256 : `validation-10.3-11.0.json`.
