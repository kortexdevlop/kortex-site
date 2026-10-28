# Étape 17.1 — Bureau virtuel Web

28/10/2026, base 10da685. Terminée et close techniquement sur Windows/Chromium.

Route /workspace et lien Bureau dans la navigation réellement détenue par Workspace.astro, sous AppLayout. Huit fenêtres au plus, dock, focus/z, glisser et redimensionnement au pointeur, déplacement Alt+flèches, taille Alt+Maj+flèches et poignée clavier. Réduction, agrandissement, restauration et fermeture. Les DOM d’iframe restent stables lors du focus ; fermer arrête le moteur et retire le canal.

Sur petit écran, onglets avec flèches/Home/End et une fenêtre visible ; aucun glisser obligatoire. Même enveloppe pastel, typographie, contours et navigation que l’accueil de référence. Le bureau est une adaptation nouvelle, absente de cette image ; aucune identité pixel avec un bureau de référence inexistant n’est revendiquée.

Chaque iframe utilise l’origine runtime distincte, sa version signée, son nonce et sa session. La disposition seule est restaurée : droits, handles et résultats ne survivent pas au rechargement. Un outil devenu obsolète doit être rouvert.

Validation : deux outils signés réels, pas de rechargement au focus, souris/clavier, minimisation/agrandissement, restauration des identifiants logiques sans doublon, onglets 390 px et absence de débordement à 320 px. Captures desktop-windows.png / desktop-mobile.png et desktop-ui.json. Astro : 92 fichiers, zéro erreur/avertissement ; build SSR réussi.

Le premier essai a révélé la clé expirée de calculatrice 1.0.0 : nouvelle publication immuable 1.0.1, construite/signée/vérifiée par le pipeline canonique et clé staging de sept jours. Aucun contournement de signature, remplacement de l’ancienne version ou changement de son contrat. Preuves publiques desktop-calculator-*.json ; politique opérateur restaurée, variables courantes préservées. Production, DNS et mail intacts.

Clôture technique documentée ; aucune approbation humaine visuelle inventée.
