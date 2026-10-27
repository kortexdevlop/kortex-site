# Étape 8.1 — CLI générique de construction

Date : 2026-10-27. Base : d6a70d8. Résultat : 5e99096. Deuxième étape du lot autorisé.

kortex-xtask découvre tools/*/manifest.json, refuse identifiants dupliqués et liens, puis résout la crate et sa cible cdylib par Cargo metadata. Aucun match par tool_id ni script Python spécifique. Build compile le target release wasm32-unknown-unknown avec --locked et limite mémoire ; validate inspecte le module existant ; package reconstruit, valide puis écrit ZIP et rapport. --all, --out, --json et --help sont disponibles. Une sortie de paquet existante de bytes différents est refusée.

| Commande / essai | Résultat |
|---|---|
| cargo run -p kortex-xtask --locked -- build --tool calculatrice --json | Réussi |
| cargo run -p kortex-xtask --locked -- build --all --out target/modules-final --json | Réussi ; module copié |
| cargo run -p kortex-xtask --locked -- validate --all --json | Réussi |
| package --tool calculatrice, puis package --all --out target/packages-second | Réussis, mêmes octets |
| package --tool calculatrice --out target/packages-final --json | Réussi, même hash après revue finale |
| Tests discovery | Deux outils arbitraires, dossiers différents des IDs ; doublon et manifeste invalide refusés |
| Tests CLI | Aide, options invalides et signature sans injection CI refusée correctement |

Les sorties JSON réussies vont sur stdout, les erreurs JSON sur stderr avec code non nul. Les noms arbitraires des fixtures de discovery vérifient l’indépendance des identifiants ; aucun sixième outil métier factice n’est ajouté au catalogue. Les sources Rust compilées doivent être de confiance, comme pour Cargo normalement.

Preuves : docs/evidence/validation-8.0-8.2.json, tests crates/kortex-xtask ; documentation complète docs/SIGNING.md. Poursuite 8.2 dans le même lot. Aucune publication distante.
