# Étape 7.1 — Calculatrice guest réelle

Date : 2026-10-27. Base : 17eb70a. Résultat : 84b0633. Première étape du lot de trois explicitement demandé par le propriétaire.

Tool traite init, render, ui.event, action.invoke et shutdown. State possède les saisies, la destination du pavé, le résultat et l’erreur. Une seule fonction calculate exécute les quatre opérations depuis UI et actions. Les entrées et sorties finies restent dans la borne du contrat, la division par ±0 produit DIVIDE_BY_ZERO. Le module utilise les exports du SDK et ne possède aucun import. Une réouverture efface les calculs précédents.

| Vérification | Résultat |
|---|---|
| cargo test -p tool-calculatrice --locked | 3 tests réussis ; table de 16 cas, parcours clavier numérique/lifecycle et 13 goldens |
| cargo build -p tool-calculatrice --target wasm32-unknown-unknown --release --locked | Réussi avec mémoire maximale de 64 Mio |
| Imports du vrai module, contrôlés par Wasmtime | 0 |
| cargo clippy -p tool-calculatrice -p kortex-runtime-native --all-targets --no-deps --locked -- -D warnings | Réussi |

L’arbre utilise les composants UI v1, sans HTML ou styles guest. Les vrais clics ont révélé un défaut du pont Web : la conversion Rust produisait des Map JavaScript au lieu des objets attendus. Correction générique de la représentation JSON, couverte par le parcours réel ; aucune branche calculatrice. La CSS partagée complète la hauteur de 44 px par une largeur minimale de 44 px.

Le design envisagé en 7.0 est précisé : le contrat UI figé ne transporte pas keydown/focus guest. Entrée/Espace sur bouton natif sont disponibles et Entrée est vérifiée ; les raccourcis globaux Entrée/Échap ne sont pas livrés. Tout effacer permet la remise à zéro. Aucun schéma public n’est modifié pour les ajouter. Détails : tools/calculatrice/design.md.

Preuves : docs/evidence/validation-7.1-7.3.json, tests/golden/calculatrice-reference.json, sources et tests tools/calculatrice. Autorisation de poursuivre 7.2 dans le même lot.
