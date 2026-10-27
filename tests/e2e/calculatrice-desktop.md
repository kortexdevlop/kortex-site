# Calculatrice — scénario Windows réel

Date d’exécution : 2026-10-27. Commande : `& tests/e2e/calculatrice-desktop.ps1`, avec CARGO_TARGET_DIR configurable. Le script construit l’hôte, prépare la fixture signée, lance uniquement son processus avec un port de débogage WebView2 temporaire et le ferme à la fin. Le produit n’active pas ce port.

1. Ouvrir target/calculatrice-fixture/calculatrice.tool avec trust.json ; annuler le dialogue hôte et vérifier qu’aucun outil n’apparaît.
2. Réouvrir, continuer sans accès sensibles, vérifier deux champs vides et « Prêt à calculer ».
3. Utiliser le pavé 2, 5, sélectionner le deuxième nombre, 4, 8 ; activer Multiplier au clavier (Entrée). Vérifier « Résultat : 1200 » et des touches d’au moins 44 × 44 px.
4. Exécuter multiply avec {a:25,b:48} dans le formulaire hôte : result=1200.
5. Saisir 0 dans le deuxième champ, cliquer Diviser : alerte DIVIDE_BY_ZERO. Action divide avec {a:25,b:0} : même code d’erreur.
6. Vérifier l’absence de requête réseau de la page pendant ces interactions ; aucun import du guest ni permission sensible dans le manifeste.
7. Fermer puis réouvrir : état vide, résultat initial, hôte toujours utilisable.

Le fichier calculatrice-desktop.cjs automatise ces assertions sur la vraie fenêtre WebView2. Capture : docs/evidence/calculatrice-windows.png. Le SHA-256 du paquet est imprimé à la fin ; il doit égaler celui du parcours Web. Linux/macOS et Firefox restent exclus sur instruction du propriétaire.
