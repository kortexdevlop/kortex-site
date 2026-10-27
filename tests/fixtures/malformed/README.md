# Corpus hostile — étape 8.3

Le corpus est généré par `cargo test -p kortex-pack --locked`, sans instancier de guest. Les recettes sont dans `crates/kortex-pack/tests/security.rs` ; `corpus.json` fixe les compteurs et le hash de référence, jamais régénéré automatiquement par un test.

Le module de départ est un petit Core WASM WAT avec les quatre exports ABI, mémoire 1–1 024 pages et aucun import. Le ZIP de départ contient le manifeste/contrat calculatrice, ce module et une icône minimale. La graine xorshift32 est `0x4b4f5254`. Les 10 000 mutations alternent ZIP/WASM ; selon l'indice modulo 5 : troncature, bit inversé, octet FF, insertion FF FF FF FF 0F, suppression. Le hash concatène pour chaque mutation sa longueur u64 little endian puis ses octets.

SHA-256 du corpus : `80a38a6e6d4985dbdab0e2e4b80d9287f0d53191dcab37f94188de0a2bd9cff1`.

| Type | Mutations | Refusées | Structure encore valide |
|---|---:|---:|---:|
| ZIP | 5 000 | 4 970 | 30 |
| WASM | 5 000 | 4 982 | 18 |

Les 48 mutations encore valides ne sont pas des défauts : modifier certains octets n'invalide pas nécessairement la structure. Elles ne conservent pas la preuve SHA/signature du paquet initial. Aucun des 10 000 candidats n'est exécuté ; zéro panic.

Cas dirigés : fichier supplémentaire, ABI incorrecte, conflit manifeste/contrat, fichier >20 Mio, WASI, mémoire sans maximum/partagée/>1 024 pages, mauvaise signature d'export. Les tests d'archives préexistants ajoutent traversal, liens, doublons, CRC et en-têtes incohérents.

Pour vérifier les frontières des hôtes, `cargo test -p kortex-runtime-native --test package_security --locked` produit `target/malformed-admission.json` : six ZIP malformés **correctement signés** par une clé TEST publique, refusés par l'admission native avant création du moteur. `web-adapter/tests/package-security.spec.ts` réutilise exactement ces octets, vérifie d'abord leur signature, puis exige six refus et zéro création de Worker. Aucune graine privée n'est écrite ; les archives générées restent sous target/.
