# Étape 7.0 — Déclaration de la calculatrice

Date : 2026-10-27. Base : 9c3e2d3. Résultat : commit contenant ce rapport. Troisième étape du lot autorisé. Prérequis : backend 6.2 et vérification Windows 6.3, sous dérogation multi-OS consignée.

La crate tool-calculatrice contient le manifeste calculatrice 1.0.0, le contrat des actions add/subtract/multiply/divide, une icône SVG, un design de clavier numérique accessible et seize cas de référence. Zéro permission sensible, side_effect=none, actions idempotentes. Les schémas bornent les nombres et résultats finis à l’intervalle sûr JSON/JavaScript. La division par ±0 produit DIVIDE_BY_ZERO ; les valeurs/résultats invalides produiront INVALID_INPUT. NaN, Infinity et mauvaises formes sont refusés avant le guest.

Le design précise champs étiquetés, noms accessibles, focus, cibles de 44 px, résultat annoncé, erreurs visibles, touches numériques, Entrée et Échap. Clic et action.invoke partageront les fonctions du guest, sans branche spéciale hôte.

| Commande | Résultat, 2026-10-27 |
|---|---|
| `pnpm --filter @kortex/contracts test` | 0 ; 3 tests, 46 cas communs Rust/TypeScript |
| `node --test tools/calculatrice/tests/declaration.test.mjs` | 0 ; 3 tests : déclaration, formes des 16 cas et limites numériques |
| `cargo check -p tool-calculatrice --target wasm32-unknown-unknown --locked` | 0 |

Les cas décrivent les résultats à implémenter ; cette étape ne les exécute pas comme calculs du guest. La crate expose uniquement les déclarations et ne contient pas de handler factice, d’export ABI ni de paquet .tool exécutable. L’implémentation et la construction WASM relèvent de 7.1, qui n’est pas démarrée. L’architecture est mise à jour. Aucun schéma gelé modifié.

Preuves : tools/calculatrice, tests/declaration.test.mjs et docs/evidence/validation-6.2-7.0.json. Le contrôle locked a été effectué après résolution de la nouvelle crate locale et inscription dans Cargo.lock.
