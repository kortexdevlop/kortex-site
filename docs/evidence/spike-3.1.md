# Preuve 3.1 — même binaire Core WASM sous Wasmtime

23 octobre 2026. Windows x64, Rust/Cargo 1.99.1, Wasmtime 50.0.0.
Binaire utilisé : tests/spikes/core-guest/target/wasm32-unknown-unknown/release/kortex_core_guest_spike.wasm, construit en 3.0. Aucune recompilation guest.
SHA-256 attendu : 00591ffa2f5faa8398585cd5068058b46efe5409f8ae6011a361844b907793f8. Le harness refuse toute autre empreinte avant instanciation.

Reproduction :

```powershell
pnpm exec playwright test tests/spikes/browser-runner/test.spec.ts --workers=1 --reporter=line --output=target/playwright-spike-3.1
cargo run --manifest-path tests/spikes/native-runner/Cargo.toml --release --locked
cargo test --manifest-path tests/spikes/native-runner/Cargo.toml --locked
```

Le premier passage enregistre les réponses navigateur aux 30 cas de wire-cases.json sous target/spike-3.1/{chromium,webkit}.json. Le natif compare les réponses complètes, pas seulement les codes de succès. Les captures sont locales et reproductibles ; elles ne sont pas remplacées par des réponses synthétiques si absentes.

Le harness utilise exactement l'import ABI kortex_v1.host_enqueue, et non le nom host_request employé comme raccourci dans le travail de l'étape. Import refusant les capacités, aucun WASI. Signatures d'exports typées vérifiées ; mémoire 32 bits avec maximum 1024 pages et limite hôte de 64 MiB.

Trois appels avec pointeur/longueur invalides vérifient le rejet guest avant lecture. Pour l'interruption CPU, un module WAT adversarial distinct contient une boucle infinie : il doit lever Trap::OutOfFuel après budget de 10000 unités. Ce module de test ne remplace ni ne modifie le guest 3.0. Fuel n'est pas une mesure de temps mural.

Résultats navigateur observés : deux réussites, Firefox omis par décision explicite ; 30 cas par moteur et 10000 multiplications supplémentaires par moteur.
Résultats natifs : à compléter après fin de compilation.

## Résultats natifs finaux

Exécution réussie (code 0) : 30 réponses complètes égales à Chromium et WebKit, trois rejets de pointeurs/longueurs, Trap::OutOfFuel effectif sur boucle infinie. Mémoire guest : 1179648 octets. Durée observée du processus entier : 3948 ms (inclut compilation JIT, lecture des captures et module adversarial ; pas une latence de dispatch).
Premier lancement : STATUS_STACK_OVERFLOW sur le thread principal Windows. Correction du banc uniquement : thread nommé wasmtime-spike avec pile native de 32 MiB, sans changement du guest ni de l'ABI. La pile sera à réévaluer dans le runtime produit. Aucun diagnostic attribué arbitrairement au guest.

Trace finale :

```json
{"browsers_compared":["chromium","webkit"],"cpu_interrupted":true,"memory_bytes":1179648,"pointer_cases":3,"sha256":"00591ffa2f5faa8398585cd5068058b46efe5409f8ae6011a361844b907793f8","wasmtime":"50.0.0","wire_cases":30}
```

Log local .git/probe/native-3.1.log, SHA-256 F1EE3B85AB881D29B1ED42A4B282C17B53EB6A8A142DC89545B67174DA7EAFF2. Firefox omis sur instruction du propriétaire ; Linux/macOS omis sur instruction antérieure. Preuve limitée à Wasmtime Windows et Chromium/WebKit.

## Reprise CR-2026-001 — 23 octobre 2026

Historique ci-dessus conservé. Suite à correction autorisée du codec, nouveau guest SHA-256 3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65, remplaçant pour les nouveaux essais 00591ffa2f5faa8398585cd5068058b46efe5409f8ae6011a361844b907793f8. Le code du guest et ses signatures ABI ne changent pas ; la crate codec liée a été reconstruite. Chromium/WebKit : 33 cas wire communs et 10000 multiplications par moteur passent. Wasmtime : mêmes octets, 33 réponses complètes identiques, trois rejets de pointeur et interruption OutOfFuel confirmés. Firefox toujours omis sur décision propriétaire. Le nouveau corpus rejette capability.result incomplet avant dispatch métier. Voir cr-2026-001.md pour résultats consolidés.
