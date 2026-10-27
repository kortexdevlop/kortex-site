# Étape 6.0 — Moteur natif Wasmtime

Finalisation : 2026-10-27. Base : ceac36a ; résultat : 02f6736 (implémentation du lot). Étape comprise dans le lot explicitement demandé par le propriétaire. Prérequis : hôte Web signé et isolé 5.4.

`kortex-runtime-native` implémente GuestEngine avec un Engine/Store par outil, acteur séparé, mémoire limitée à 64 Mio, une mémoire, une table bornée à 65 536 éléments, fuel et interruption epoch. Seul import autorisé : kortex_v1.host_enqueue. Aucun WASI ni bibliothèque native d’outil. Exports et types mémoire contrôlés. Les demandes host_enqueue sont refusées ; les pilotes natifs relèvent de 6.2.

L’admission vérifie SHA-256 et Ed25519 strict avant extraction, puis manifest/contract et leur correspondance avec VerifiedTool. Extraction ZIP uniquement en mémoire : archive 25 Mio, 256 entrées, 20 Mio par entrée, total décompressé 50 Mio, ratio 100, chemins bornés, doublons et liens refusés. Métadonnées revérifiées à l’instanciation. Buffers ABI bornés, non chevauchants, puis libérés ; corrélation vérifiée. Une erreur fatale ferme l’acteur. Drop et annulation interrompent son moteur.

| Commande | Environnement | Résultat |
|---|---|---|
| `cargo test -p kortex-runtime-native --locked` | Windows, 2026-10-23 | 2 tests natifs réussis |
| `cargo test -p kortex-runtime-native -p kortex-abi --locked` | Windows | 2 tests natifs + 9 ABI |
| `cargo clippy -p kortex-runtime-native -- -D warnings` | Windows | Sortie 0, commande complète |
| `cargo test -p kortex-capabilities -p kortex-core -p kortex-shared --locked` | Windows, 2026-10-23 | 36 tests réussis |

Le module exact de 3.0 a pour SHA-256 `3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65`. Il effectue 100 multiplications natives 25 × 48 donnant 1200. Les modules adverses démontrent l’interruption d’une boucle et le rejet d’un buffer invalide, avec erreurs Timeout et InvalidReply déterministes en moins de 5 secondes.

Clippy a initialement révélé des avertissements collapsible_if préexistants dans ABI, capacités, validateurs partagés et Core. Les conditions ont été regroupées sans changer leur sémantique ; les régressions passent et les cinq schémas gelés gardent leurs empreintes. VerifiedTool devient clonable en conservant ses champs privés et son immutabilité.

Limites du prototype : compilation sur l’acteur sans garantie d’interruption du compilateur ; bornes d’exécution prouvées après compilation. L’attente des réponses utilise un réveil actif à optimiser. Les tests prouvent les limites d’exécution ; aucun test exhaustif des courses d’annulation n’est revendiqué.

Preuves : `crates/kortex-runtime-native/tests/guest.rs`, `docs/evidence/native-engine.md`, `docs/evidence/validation-5.4-6.1.json`. Linux/macOS et Firefox exclus selon instruction.
