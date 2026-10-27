# Scénario natif — 6.3

Le propriétaire a exclu Linux/macOS et Firefox. Seul Windows est validé localement ; aucune preuve Windows ne vaut validation des autres OS. Le gate global multi-OS reste non satisfait. La poursuite vers 7.0 utilise la dérogation consignée dans docs/DECISIONS.md.

Sur Windows, depuis la racine :

```powershell
$env:CARGO_PROFILE_DEV_DEBUG='0'
$env:CARGO_PROFILE_TEST_DEBUG='0'
$env:CARGO_INCREMENTAL='0'
# Facultatif : CARGO_TARGET_DIR vers un disque disposant de place.
cargo test -p kortex-runtime-native -p kortex-capabilities -p kortex-core --locked
powershell -NoProfile -ExecutionPolicy Bypass -File tests/desktop/native-smoke.ps1
cargo test -p kortex-runtime-native --locked --test capabilities live_https -- --ignored
```

Le script construit l’application, génère un .tool signé par clé éphémère, lance la vraie WebView2 avec un port CDP temporaire, vérifie calcul, consentement, fermeture, erreurs et récupération, puis ferme le processus. Le produit n’active pas CDP par défaut. Ne pas lancer en parallèle d’un autre test écoutant sur 9332. Aucun secret ni clé privée de distribution n’est utilisé.

Contrôles automatisés :

- Même checksum du guest Core, 100 multiplications à 1200 ; boucle et annulation interrompues.
- Guest SDK réel : demande storage.read, refus/accord, capability.result, UI identique au scénario Web, fermeture et réouverture.
- Stockage persistant : isolation, quota, révision atomique, migration/rollback, corruption et jonctions refusées.
- Réseau : DNS validé par Gateway, IP publiques épinglées, HTTPS/TLS, proxy désactivé, réponse bornée ; redirection externe refusée. Le test Internet explicite ne tourne pas dans la suite hors ligne.
- Application : JSON invalide et archive altérée visibles ; module signé incompatible refusé ; ouverture suivante réussie sans redémarrer l’application.

Sélecteurs OS : le code utilise rfd et les backends testent explicitement la sélection vide/annulation. L’ouverture et l’annulation interactives de la fenêtre OS ne sont pas automatisées par ce scénario. Le presse-papiers est raccordé via arboard mais aucun test ne modifie le presse-papiers réel de l’utilisateur.

Pour une validation Linux future : compiler/tester nativement avec les dépendances WebKitGTK du backend Dioxus/Wry de la version verrouillée, lancer le shell avec le même ZIP et comparer UI/actions/permissions. Pour macOS : compilation native avec SDK Apple et WKWebView, même ZIP sans recompilation guest et même scénario. Ces lignes sont une checklist future, pas une preuve de réussite. Les runners et installateurs relèvent de 19 ; la certification des trois OS du gate 20 reste ouverte.
