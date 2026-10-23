# Étape 2.4 — Contrats partagés TypeScript/Rust

23 octobre 2026. Windows, Node 24.15.0, Rust 1.99.1. Base 728f57a.
Prérequis 2.3 approuvé par « Oui continue ».
Créations : shared/typescript/package.json, src/index.ts, src/validate.ts ; shared/rust/Cargo.toml, src/lib.rs ; shared/generated/README.md ; tests/contracts/cross-language.test.mjs ; ce rapport.
Modifications : Cargo.toml, Cargo.lock, pnpm-lock.yaml, PROGRESS.md. pnpm-workspace.yaml référençait déjà shared/typescript : aucune modification nécessaire.

TypeScript : types publics, validateManifest/validateContract/validateUi/validateWorkflow retournant Result discriminé, violations SCHEMA/INVARIANT/LIMIT sans valeurs d'entrée. Rust : modèles serde et API validate_manifest/validate_contract -> Result<model, Vec<Violation>> ; jsonschema 0.60.0 épinglé, rust-version déclarée 1.85, compilé avec 1.99.1 ; résolution HTTP et fichiers désactivée.

Même source JSON Schema dans les deux langages. Compléments sémantiques pour identifiants et bornes. Limite documentaire de 1 MiB appliquée aux entrées de validation partagée. Types maintenus explicitement ; pas de prétendue génération de toutes les déclarations depuis le schéma. Régénération vérifiée : compilation runtime depuis les schémas, instantané diagnostique déterministe dans target/contracts ; deux écritures identiques, hashes source inchangés.

Résultats observés :
- pnpm install --lockfile-only puis pnpm install --frozen-lockfile : code 0.
- pnpm --filter @kortex/contracts typecheck : code 0.
- pnpm --filter @kortex/contracts test : trois tests réussis, matrice commune de 33 cas TS/Rust identiques, code 0. Le test lance effectivement le validateur Rust via cargo et compare son tableau de résultats.
- cargo test -p kortex-shared --locked : deux tests réussis, code 0 ; le test de matrice externe reçoit ses cas du test Node, les fixtures canoniques s'exécutent également seules.
- node scripts/validate-schema.mjs : 41 contrôles réussis, code 0.
- cargo fmt --all : code 0.

Limites : matrice croisée couvre manifeste/contrat conformément aux API Rust de l'étape ; UI/workflows vérifiés côté TS, pas de validateur Rust de ces documents revendiqué. Erreurs Rust pointent la racine, TS utilise les chemins AJV : catégories communes, chemins détaillés non identiques. L'instantané généré ne remplace pas la source normative. Aucun essai Linux/macOS. Aucun gel.
Statut awaiting_approval. Commit résultat : commit contenant ce rapport.

Contrôle final : deuxième exécution des tests croisés réussie (code 0), sans changement des schémas ou fichiers générés suivis. Log local .git/probe/contracts-2.4.log, SHA-256 D2B63AD988A841C3BA49BD97BF507D9309AC36028AD81F72B9B318CC9F86042A. node scripts/check-progress.mjs : code 0 ; six tests de secrets réussis ; git diff --cached --check : code 0.
