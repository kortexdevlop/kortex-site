# Étape 7.3 — Référence de compatibilité outil 1

Date : 2026-10-27. Base : 17eb70a. Résultat : 84b0633. Troisième et dernière étape du lot demandé.

Treize paires Request/Reply sous tests/golden sont figées avec les empreintes du guest, du ZIP de test et du corpus. Les tests les exécutent sur le vrai SDK, le Worker Web et Wasmtime. Le contrôle Node ajoute vérification des hashes et signature du paquet. Le guide TOOL_SDK.md décrit la création d’un outil par la chaîne standard et les limites de cette référence ; CONTRACTS.md référence les nouveaux tests sans modifier l’ABI.

| Vérification finale | Résultat |
|---|---|
| cargo test --workspace --locked | 80 tests réussis dont 1 doctest ; 0 échec ; 1 smoke HTTPS externe explicitement ignoré |
| pnpm --filter @kortex/web-adapter test | 28 tests réussis Chromium/WebKit |
| pnpm exec playwright test tests/e2e/calculatrice-web.spec.ts tests/web | 10 tests réussis, dont 2 calculatrice et 8 régressions shell/iframe/focus |
| node --test tools/calculatrice/tests/declaration.test.mjs tests/fixtures/calculatrice.test.cjs | 4 tests réussis |
| pnpm --filter @kortex/contracts test | 3 tests réussis, 46 cas communs Rust/TypeScript |
| pnpm --filter @kortex/web-adapter typecheck | Réussi |
| Cinq hashes de schémas gelés et git diff --check | Inchangés / réussi |

Le contrôle de l’outil ne requiert aucune branche par identifiant, extension de schéma ou nouveau mécanisme d’admission. Deux corrections hôtes transversales sont explicitement documentées : objets JSON au pont Web et largeur minimale des boutons. Le gate ne prétend pas qu’aucun fichier hôte n’a changé.

Gate détaillé : PHASE7_GATE.md. Preuves : docs/evidence/validation-7.1-7.3.json et tool-1-parity.md. Gel local autorisé par la demande du lot ; pas d’attestation humaine ou sécurité indépendante inventée. La phase 8 n’est pas démarrée ; les exclusions OS/navigateur restent valables.
