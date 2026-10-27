# Matrice des cinq outils — étape 12.5

Généré le 2026-10-27T22:30:12.087Z à partir de docs/evidence/five-tools-web.json, phase12-windows.json et phase12-staging.json.

| Outil | Version | SHA paquet | SHA guest (module.wasm) | Permissions déclarées | Actions vérifiées | Web (Chromium, paquet signé) | Bureau Windows (WebView2) | Publié staging |
|---|---|---|---|---|---|---|---|---|
| calculatrice | 1.0.0 | `1d4c1a0967b5` | `f095744f63b9` | aucune | add, divide | oui | calculatrice : preuve 7.x/9.x | oui |
| convertisseur-casse | 1.0.0 | `d4dfadd3c099` | `7c0345eb1d2e` | aucune | upper, title | oui | oui | oui |
| calculateur-hash | 1.0.0 | `aee5fc4a0e71` | `701b38100ea3` | file.pick, file.read | sha256_bytes | oui | oui | oui |
| timestamp-utc | 1.0.0 | `7ca4588ad879` | `dbd029d4ea26` | aucune | from_unix, to_unix | oui | oui | oui |
| explorateur-csv | 1.0.0 | `183f838f5548` | `0b8561e2b1da` | file.pick, file.read, file.save, storage.read, storage.write | summarize_csv, export_report | oui | oui | **non — quota D1** |

## Lecture

- **Mêmes valeurs** : les 9 vecteurs de tests/integration/five-tools-vectors.json sont exécutés par le test Web (Playwright, `five-tools.spec.ts`, Chromium via Chrome système) et par le test Wasmtime Windows (`five_tools_produce_the_shared_web_vectors_on_the_native_wasmtime_host`). Les deux comparent les sorties exactes ; 9/9 réussis des deux côtés.
- **Même binaire guest** : les deux hôtes chargent la même archive signée (même SHA paquet vérifié par le descripteur) ; le SHA du `module.wasm` est calculé depuis cette archive. Le bureau ne recompile pas l'outil.
- **Permissions** : les trois outils sans permission (calculatrice, convertisseur de casse, dates UTC) ne déclenchent aucun dialogue de consentement après exécution de toutes leurs actions (contrôle Web). Le calculateur de hash et l'explorateur CSV ne demandent leurs permissions que sur clic.
- **Aucun contournement par identifiant d'outil** : recherche des cinq identifiants dans crates/kortex-core, kortex-runtime-native, kortex-ui, kortex-capabilities, kortex-abi, kortex-sdk, kortex-verify, kortex-pack, runtime, web-adapter/src, api/src, database/migrations et shared : aucun résultat.

## Écarts et limites

- **Outil 5 non publié sur le staging** : la publication est arrêtée par `D1_ERROR: exceeded D1's free tier daily row read limit`. Aucune ressource n'a été écrite et la politique d'administration temporaire a été restaurée (docs/evidence/phase12-csv-staging.json). Le catalogue public liste quatre outils. Cette ligne reste **non satisfaite** tant que la publication n'est pas rejouée.
- **WebKit non exécuté** pour cette matrice : aucun navigateur Playwright n'est installé sur cette machine (un téléchargement n'a pas été demandé). Les essais WebKit de 12.0 à 12.2 restent ceux de docs/evidence/phase12-local-playwright.json.
- **Stockage Web** : le harnais local n'a pas d'origine d'exécution dédiée ; il démontre l'échec fermé. Le stockage réel est démontré côté bureau (Wasmtime : écriture, refus, relecture après redémarrage). Le test Web sur origine dédiée dépend de la publication staging.
- **Dialogue natif de sauvegarde/sélection Windows** non automatisé : le refus de `file.save` est testé, pas l'écriture positive.
- Calculatrice : preuve Windows héritée des phases 7 à 9 (docs/evidence/calculatrice-windows.png) ; non rejouée dans WebView2 ici.
