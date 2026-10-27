# Étape 8.2 — SHA-256, Ed25519 et confiance publique

Date : 2026-10-27. Base : d6a70d8. Résultat : commit contenant ce registre. Troisième et dernière étape du lot demandé.

kortex-verify expose verify_archive_hash et verify_descriptor sur le descripteur B.2. Le texte exact terminé par LF, les SHA minuscules et les signatures base64 canoniques sont communs à Rust et au Web. artifact.schema.json formalise les sept champs et refuse les inconnus ; le Web utilise sa version AJV précompilée, compatible CSP. Le chargeur sépare son URL de téléchargement de ce descripteur.

SignedKeys contient un snapshot versionné, fenêtres temporelles et clés publiques, signé par une racine hors ligne épinglée. Clé absente, retirée, révoquée, expirée, racine incorrecte, document modifié et version inférieure au minimum hôte sont refusés. Le CLI propose sign/verify, n’accepte la graine que par environnement CI et n’en imprime pas la valeur. Les tests utilisent des secrets TEST en mémoire et vérifient l’absence de fuite en sortie ; la configuration applicative refuse également la nouvelle variable de graine.

| Vérification | Résultat |
|---|---|
| Tests kortex-verify | 4 tests réussis ; mutations de bytes, id, version, ABI, signature et politiques de clés |
| Tests kortex-xtask | 3 discovery/CLI + 1 signature/vérification avec injection TEST réussis |
| cargo run -p kortex-xtask --locked -- verify --artifact tests/fixtures/artifact-signed-test.json --json | Réussi, signature_verified=true, trust_profile=test-only-default |
| Fixture publique commune Rust/Web | ZIP canonique vérifié ; multiply=1200 dans Chromium et WebKit |
| Mutations et rotations côté navigateurs | 12 refus attendus par navigateur ; signatures originales acceptées |
| cargo test --workspace --locked | 94 réussis dont 1 doctest, 0 échec ; 1 smoke HTTPS externe explicitement ignoré |
| pnpm --filter @kortex/web-adapter test | 30 tests réussis |
| pnpm exec playwright test tests/e2e/calculatrice-web.spec.ts tests/web | 10 tests réussis |
| Typecheck, Clippy trois nouvelles crates, contrôle environnement et secrets | Réussis |
| Référence TOOL1_V1 et cinq schémas gelés | Inchangés |

Le générateur TEST n’écrit que sous target/ et ne persiste aucune clé privée. Le dépôt reçoit seulement le paquet, son descripteur et les documents publics root/keys ; il ne possède aucune clé de production. SIGNING.md explique la rotation et la récupération. Le minimum de version doit être conservé par l’hôte ; le CLI ne le persiste pas, et aucune distribution de révocation distante n’est revendiquée. Les anciens chemins natifs de confiance ne sont pas silencieusement remplacés dans ce lot.

Les étapes 8.3–8.5, l’orchestration de publication locale et le gel PACKAGE_FORMAT_V1 restent à faire. Aucun faux gate phase 8, aucun déploiement ni signature humaine/sécurité indépendante n’est déclaré. Linux/macOS et Firefox restent exclus. Preuve structurée : docs/evidence/validation-8.0-8.2.json.
