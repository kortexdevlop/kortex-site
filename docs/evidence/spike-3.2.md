# Prototype Dioxus Web/Desktop — preuve 3.2

23 octobre 2026. Windows x64, Rust 1.99.1, Dioxus/dx 0.7.10, web-sys 0.3.106, Chrome for Testing 156.0.8078.4, WebView2 Edge 155.0.4283.62.

Le même src/main.rs charge shared/fixtures/ui.valid.json à la compilation dans les deux hôtes. Il rend column, heading, input, button et svg-chart du fixture. Carte enveloppante et table de données A=2/B=4 sont des composants hôtes supplémentaires du banc, sans modification du fixture. Prototype de faisabilité, pas encore renderer complet des 18 types.

Le CSS est inclus depuis assets/style.css dans le code hôte. Le graphique SVG utilise des éléments hôtes construits depuis labels/values ; ni HTML brut, ni script, ni accès DOM ne sont introduits dans le guest Core WASM. Le guest et son binaire ne sont pas modifiés dans cette étape. Ce prototype UI ne passe pas encore les événements par le dispatch guest : cette intégration appartient au runtime.

## Reproduction observée

Le cache natif partagé des essais est situé dans .git/probe/rust/target ; le cache Web utilisé par dx est local au prototype, dans .git/probe/rust/target sous son dossier. Ces caches restent hors Git.

```powershell
# Depuis la racine
$env:CARGO_TARGET_DIR=Join-Path (Get-Location) '.git/probe/rust/target'
cargo check --manifest-path tests/spikes/dioxus-render/Cargo.toml --locked
cargo build --manifest-path tests/spikes/dioxus-render/Cargo.toml --locked

# Depuis tests/spikes/dioxus-render
$env:CARGO_TARGET_DIR=Join-Path (Get-Location) '.git/probe/rust/target'
dx build --platform web --no-default-features --features web --debug-symbols false

# Retour à la racine : lancement de la seule application de test
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS='--remote-debugging-port=9332 --remote-allow-origins=*'
Start-Process -FilePath (Resolve-Path '.git/probe/rust/target/debug/kortex-dioxus-render-spike.exe') -WindowStyle Hidden
Remove-Item Env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
$env:KORTEX_WEB_DIST=Join-Path (Get-Location) 'tests/spikes/dioxus-render/.git/probe/rust/target/dx/kortex-dioxus-render-spike/debug/web/public'
node tests/spikes/dioxus-render/check.cjs
```

Le port CDP de test 9332 est local et n'est pas configuré dans une application distribuée. check.cjs sert le build Web en boucle locale sur port éphémère, vérifie les deux hôtes et écrit les captures ; fermer ensuite l'application de test. Une WebView native ne supporte pas le rechargement CDP comme une page Web ordinaire dans ce banc : le contrôle final n'utilise pas reload.

Résultats : cargo check code 0 ; cargo build Desktop code 0 ; dx build Web code 0. Premier échec aria_label SVG corrigé par attribut explicite aria-label. Premier build Web échoué sur Window.location dans dioxus-web ; ajout explicite de web-sys/Location, sans modifier la dépendance amont ni l'ABI.

```text
Desktop WebView assertions PASS
Chrome assertions and Desktop/Web semantic parity PASS
```

Assertions : titres et labels visibles, présence de table et graphique accessible, saisie Search répercutée dans output, focus input et bouton, clic incrémentant le compteur. Comparaison exacte des textes de main, clés data-key et géométrie des barres après normalisation du compteur cumulatif (l'incrément est vérifié séparément). Ce test n'est pas une comparaison pixel à pixel ni un audit complet d'accessibilité.

Captures inspectées visuellement : mêmes labels, hiérarchie, couleurs et proportions des composants. Chrome 1000x900 ; WebView 784x861, donc marges de page différentes selon largeur disponible. Le compteur reflète les clics cumulés de chaque session. Les deux rendus restent lisibles sans chevauchement.

- spike-3.2-chrome.png : SHA-256 970AF76A350E33CB38C9AFF23969AE3FDBBC446042E103C06F481D0637BD4BC1.
- spike-3.2-desktop.png : SHA-256 A7ACBD50BEA82466B826A1E7387E7FB4B9F8292CC132BCEC1D7475DBCF9CC911.

Linux/macOS et Firefox restent omis conformément aux décisions précédentes. Preuves P2/P4 locales Windows sur le sous-ensemble du fixture ; aucun gel global ni renderer produit complet revendiqué.
