# Kortex Desktop expérimental — Windows

Prérequis : environnement Rust/MSVC du projet et Microsoft Edge WebView2 installé.

Depuis la racine, générer la fixture signée puis lancer le shell :

```powershell
node tests/fixtures/make-tool.cjs
cargo run -p kortex-runtime-desktop --locked -- target/desktop-fixture/demo.tool target/desktop-fixture/trust.json
```

Choisir Ouvrir, puis « Continuer sans accès sensibles ». Le renderer affiche Ready. L’action multiply avec `{"a":25,"b":48}` donne `{"result":1200}`. Fermer détruit l’instance. Pour un autre paquet, fournir le .tool, son descripteur signé `<paquet>.tool.json` et un fichier JSON hôte associant identifiant de clé à clé publique Ed25519 base64. Aucun mode non signé. Le générateur ne conserve aucune clé privée et ses résultats sont ignorés par Git.

Le dialogue initial ne confère aucun privilège. Les demandes du guest sont ensuite présentées avec capacité, portée et durée ; les accès non déclarés sont refusés. Fichiers : sélection OS obligatoire, handles opaques et lecture par blocs. Stockage : données non secrètes sous le répertoire local OS `kortex/tools/<tool_id>`, profils isolés et quota de 5 Mio. Réseau : aucune origine autorisée par défaut ; un troisième argument CLI peut désigner une configuration hôte JSON `{"network_origins":["https://example.com"]}`. Les adresses DNS privées, redirects non autorisés et HTTP sont refusés. Le presse-papiers nécessite également un accord et un geste hôte récent.

La WebView rend l’interface ; le module s’exécute dans Wasmtime. Pour reproduire le test Windows avec diagnostic WebView2 temporaire :

```powershell
cargo build -p kortex-runtime-desktop --locked
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9332 --remote-allow-origins=*'
$testApp = Start-Process -FilePath (Resolve-Path 'target/debug/kortex-runtime-desktop.exe') -ArgumentList @('target/desktop-fixture/demo.tool','target/desktop-fixture/trust.json') -WindowStyle Hidden -PassThru
Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
try { node tests/native/desktop.cjs } finally { Stop-Process -Id $testApp.Id -ErrorAction SilentlyContinue }
```

Lancer depuis la racine. Le produit n’active pas ce port par défaut. Les validations ont utilisé CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0 et CARGO_INCREMENTAL=0 pour limiter les caches de développement.

Le scénario complet, avec consentement guest et récupération après erreur, est exécutable par `tests/desktop/native-smoke.ps1`. La checklist multi-OS et les limites des tests OS figurent dans `tests/desktop/native-smoke.md`. Les essais Linux/macOS sont exclus par décision du propriétaire, sans déclaration de compatibilité correspondante.
