# Kortex Desktop expérimental — Windows

Prérequis : environnement Rust/MSVC du projet et Microsoft Edge WebView2 installé.

Depuis la racine, générer la fixture signée puis lancer le shell :

```powershell
node tests/fixtures/make-tool.cjs
cargo run -p kortex-runtime-desktop --locked -- target/desktop-fixture/demo.tool target/desktop-fixture/trust.json
```

Choisir Ouvrir, puis « Continuer sans accès sensibles ». Le renderer affiche Ready. L’action multiply avec `{"a":25,"b":48}` donne `{"result":1200}`. Fermer détruit l’instance. Pour un autre paquet, fournir le .tool, son descripteur signé `<paquet>.tool.json` et un fichier JSON hôte associant identifiant de clé à clé publique Ed25519 base64. Aucun mode non signé. Le générateur ne conserve aucune clé privée et ses résultats sont ignorés par Git.

Le dialogue ne confère pas de privilège au guest : les pilotes natifs relèvent de 6.2. La WebView rend l’interface ; le module s’exécute dans Wasmtime. Pour reproduire le test Windows avec diagnostic WebView2 temporaire :

```powershell
cargo build -p kortex-runtime-desktop --locked
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9332 --remote-allow-origins=*'
$testApp = Start-Process -FilePath (Resolve-Path 'target/debug/kortex-runtime-desktop.exe') -ArgumentList @('target/desktop-fixture/demo.tool','target/desktop-fixture/trust.json') -WindowStyle Hidden -PassThru
Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
try { node tests/native/desktop.cjs } finally { Stop-Process -Id $testApp.Id -ErrorAction SilentlyContinue }
```

Lancer depuis la racine. Le produit n’active pas ce port par défaut. Les validations ont utilisé CARGO_PROFILE_DEV_DEBUG=0, CARGO_PROFILE_TEST_DEBUG=0 et CARGO_INCREMENTAL=0 pour limiter les caches de développement.
