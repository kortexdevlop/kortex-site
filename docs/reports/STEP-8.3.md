# Étape 8.3 — Refus des paquets hostiles

Date : 2026-10-27. Base : 02b0f6d. Résultat : 8ffb37d. Prérequis : 8.2 validée. Autorisation du lot : propriétaire, « Enchaîne les 4 prochaines étapes » ; exclusions Linux/macOS et Firefox maintenues.

L'inspection WASM est isolée dans wasm_check.rs et intégrée au validateur. Elle refuse imports/exports non autorisés, WASI/start, mémoire partagée/non bornée et signatures ABI incorrectes avant instanciation. Le packageur contrôle aussi la stabilité de l'identité entre discovery/build/collect. Corpus, recettes et empreintes : tests/fixtures/malformed. Les dépendances ajoutées sont locales ou déjà verrouillées ; Cargo.lock ne change que leurs arêtes.

| Commande | Date / environnement | Sortie | Résultat |
|---|---|---:|---|
| cargo test --workspace --locked | 2026-10-27, Windows, Rust verrouillé | 0 | Inclut kortex-pack et kortex-verify ; 10 000 mutations sans panic ni instanciation |
| cargo test -p kortex-runtime-native --test package_security --locked | même environnement | 0 | 6 archives malformées correctement signées refusées ; racine et altérations contrôlées |
| pnpm exec playwright test -c tests/web/playwright.config.ts | Windows, Chromium/WebKit | 0 | Corpus signé commun refusé, zéro Worker créé |
| cargo clippy -p kortex-pack -p kortex-verify -p kortex-xtask --all-targets --locked -- -D warnings | Windows | 0 | Aucun avertissement dans les trois crates |

9 952 mutations sont structurellement refusées, 48 restent valides structurellement. Aucune n'est exécutée ; la structure ne remplace pas la preuve cryptographique. Les tests cryptographiques refusent les changements de hash/signature/identité et les politiques de clés invalides. Les essais ne sont ni fuzzing exhaustif ni audit tiers.

Fichiers hôtes complémentaires justifiés par le gate 8.5 du même lot : lecteur natif partagé, contrôle moteur avant instanciation, parseurs ZIP/ABI Web et tests d'admission. Aucun schéma public antérieur modifié. Preuves et hashes de logs : docs/evidence/validation-8.3-9.0.json ; détail : docs/evidence/pack-security.md. Critère local satisfait.
