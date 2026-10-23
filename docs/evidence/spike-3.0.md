# Preuve du prototype 3.0 — 23 octobre 2026

Environnement : Windows x64, Rust 1.99.1, Node 24.15.0, Playwright 1.64.1. Chromium 156.0.8078.4 (build 1248), Firefox 157.0 (1555), WebKit 27.2 (2370).

Construction reproductible PowerShell :

```powershell
$env:RUSTFLAGS='-C link-arg=--max-memory=67108864'
cargo build --manifest-path tests/spikes/core-guest/Cargo.toml --target wasm32-unknown-unknown --release --locked
Remove-Item Env:RUSTFLAGS
pnpm exec playwright test tests/spikes/browser-runner/test.spec.ts --workers=1 --reporter=line --output=target/playwright-spike-3.0
```

Le lockfile isolé du guest est versionné. Le binaire de build reste exclu de Git ; le plafond de mémoire est passé explicitement au linker. Le Worker vérifie les imports et n'accorde aucune capacité. L'unique import est kortex_v1.host_enqueue : un message probe sans capacité est rejeté avec -1 pendant init, sans exécuter d'effet. Le guest n'accède à aucun DOM ou API navigateur ; memory et les trois fonctions ABI sont exportées.

Résultats observés : build release code 0. Test Playwright code global 1 : Chromium PASS, WebKit PASS, Firefox FAIL au lancement. Chromium : 10000 appels en 222.3 ms ; WebKit : 10000 appels en 204 ms. Ces mesures locales ne sont pas un engagement de performance. Chaque appel renvoie multiply(25,48)=1200 et le request_id fourni.

Extrait :

```text
chromium {"count":10000,"ms":222.29999995231628,"allocations":0,"denied":1,"memory":1179648}
webkit {"count":10000,"ms":204,"allocations":0,"denied":1,"memory":1179648}
firefox: browserType.launch: spawn UNKNOWN
2 passed, 1 failed
```

Les deux moteurs qui démarrent passent init/render/action.invoke, 10000 appels identiques, JSON invalide, rejet avant allocation d'une entrée >1 MiB, rejet guest d'une longueur excessive, terminaison puis redémarrage du Worker et nouvelle multiplication. Copies de réponse avant dealloc et libération des deux allocations à chaque appel ; le compteur de bookkeeping revient à zéro. Ce compteur ne constitue pas une preuve exhaustive d'absence de fuite de l'allocateur.

Firefox a été réinstallé avec playwright install firefox --force (code 0), mais reste en échec avant exécution du guest. Diagnostic Windows SideBySide : assembly mozglue introuvable. Aucun changement système ni contournement de protection effectué. WebKit est un essai supplémentaire, pas une substitution silencieuse à Firefox. Preuve Chromium/Firefox exigée par le plan non complète ; portabilité non gelée.

SHA-256 du module testé : 00591FFA2F5FAA8398585CD5068058B46EFE5409F8AE6011A361844B907793F8. Les essais 3.1 devront utiliser exactement ces octets, ou réidentifier explicitement un nouveau build avant comparaison.
