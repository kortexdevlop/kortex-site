# Étape 8.0 — ZIP canonique et validation bornée

Date : 2026-10-27. Base : d6a70d8. Résultat : 5e99096. Première des trois étapes demandées par le propriétaire après 7.3.

kortex-pack produit des ZIP à noms lexicaux, timestamp DOS 1980-01-01, mode Unix 0644 et compression stored stable. Les deux documents JSON sont triés récursivement. Le lecteur inspecte tous les en-têtes avant décompression en mémoire : chemins relatifs ASCII sûrs, collisions de casse et doublons refusés, types spéciaux/liens/volumes refusés, longueur/CRC/local-central cohérents, aucun ZIP64/chiffrement/descriptor/extras/commentaire ou trou entre entrées. Les sorties réelles du décompresseur sont bornées à la taille annoncée plus un octet.

Plafonds de publication : ZIP et cumul extrait 25 Mio, fichier/WASM 20 Mio, ratio 20:1, 256 entrées. Seuls manifeste, contrat, module et icône référencée sont emballés. Les schémas partagés et invariants de permissions/identité sont appliqués. Le WASM binaire est validé statiquement : signatures ABI, mémoire unique 32 bits ≤64 Mio, import host_enqueue facultatif uniquement, pas de start. Le rapport sérialisable contient valid, issues/code/path, nombres de fichiers/octets et identité.

| Vérification | Résultat |
|---|---|
| cargo test -p kortex-pack --locked | 6 tests réussis |
| Deux constructions indépendantes via CLI | Même SHA-256 1d4c1a0967b583489f69921d80d1c062dbd57ecb0d9dabea8337575e4ab32787 |
| Malformations | Traversal, collisions/doublons, CRC, symlink/type spécial, volume DOS, en-têtes incohérents, bombe, taille et fichier non référencé refusés |
| Deflate borné et validation WASM | Cas valide accepté ; mauvais exports/imports/mémoire/WAT refusés |
| Troncatures et mutations d’en-têtes | Refus ou erreur structurée, aucune panique |

La signature est indépendante de cette bibliothèque et intervient seulement après le ZIP final. La validation d’asset porte sur présence, chemin, taille et référence, pas sur un assainissement SVG ; la validation statique WASM ne prouve ni terminaison ni résultat métier. Les nouvelles règles de publication ne sont pas présentées comme une migration de tous les anciens lecteurs hôtes. Aucun schéma gelé ne change. Le format reste candidat jusqu’à 8.5.

Preuves : docs/evidence/validation-8.0-8.2.json, tests crates/kortex-pack et paquet public de test. Poursuite 8.1 dans le lot autorisé.
