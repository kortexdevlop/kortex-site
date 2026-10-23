# Étape 3.1 — Portabilité binaire native

23 octobre 2026, base da4dac8. Poursuite après 3.0 autorisée par « Oui continue, laisse firefox de coté » ; dérogation dans DECISIONS.
Créations : native-runner/Cargo.toml, Cargo.lock, src/main.rs, tests/compat.rs, wire-cases.json ; docs/evidence/spike-3.1.md ; ce rapport.
Modifications : browser-runner/test.spec.ts pour exercer les mêmes 30 cas, capturer les réponses et omettre Firefox explicitement ; DECISIONS et PROGRESS pour la dérogation et la progression. Aucun contrat ABI modifié.
Le lock natif est isolé du workspace produit. Profil release expérimental opt-level=0 pour borner les ressources de compilation ; aucune revendication de performance optimisée.
Résultats finaux : cargo run --manifest-path tests/spikes/native-runner/Cargo.toml --release --locked : code 0. cargo test --manifest-path tests/spikes/native-runner/Cargo.toml --locked : un test intégration réussi, code 0. 30 réponses complètes égales à Chromium/WebKit ; hash exact ; trois rejets de pointeurs ; interruption OutOfFuel de la boucle infinie. Premier lancement en échec de pile Windows, corrigé par un thread natif de 32 MiB, sans toucher au guest. Statut awaiting_approval ; aucun gel. Commit résultat : commit contenant ce rapport.

Régression : Playwright deux réussites et Firefox omis, code 0 ; node tests/conformance/check.mjs : 23 cas réussis, code 0 ; cargo test -p kortex-abi --locked : sept réussites, code 0. Mesures et log hashés dans la preuve.

Reprise autorisée CR-2026-001 : nouveau binaire 3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65, Chromium/WebKit et Wasmtime retestés avec 33 cas communs. Ancien hash conservé dans la preuve originale ; résultats consolidés dans docs/evidence/cr-2026-001.md.
