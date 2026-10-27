# Gate phase 8 — PACKAGE_FORMAT_V1

Revue locale du 27 octobre 2026, Windows, base 02b0f6d. Autorisation du lot : propriétaire, « Enchaîne les 4 prochaines étapes ». Résultat : commit d'implémentation identifié dans PROGRESS.md. Décision : critères locaux satisfaits, gel PACKAGE_FORMAT_V1 et poursuite vers 9.0 dans le même lot. Ce registre ne prétend pas être une signature humaine de sécurité indépendante.

| Élément de référence | Valeur |
|---|---|
| Outil / ABI / UI | calculatrice 1.0.0 / 1.0 / 1.0 |
| ZIP canonique | 264 845 octets |
| SHA-256 ZIP | 1d4c1a0967b583489f69921d80d1c062dbd57ecb0d9dabea8337575e4ab32787 |
| SHA-256 module | f095744f63b9594af61c02a6a515f2ad7ddab0ca3d5e7ccbcc9120a78f850c50 |
| artifact.schema SHA-256 | 3712fe66d1ff069315b19e87fae71ffe7d523b9843ff1caed02632af84f7926c |
| Preuve | sept champs B.2, signature Ed25519 détachée, SHA du ZIP final |
| Confiance | racine hôte épinglée TEST + SignedKeys v1 signé, dates/révocations/minimum |

Le pipeline générique produit les mêmes octets dans deux stagings indépendants et conserve UUID/signature à répétition d'un staging. Le ZIP canonique correspond à la fixture publique déjà signée par une ancre TEST permanente ; le gate UI utilise la preuve éphémère produite par le pipeline, pas une signature substituée. Les ancres sont de la configuration opérateur des hôtes, jamais un fichier autorisant le guest à se signer lui-même.

Chromium et WebKit chargent ce ZIP par le ToolLoader normal dans le shell Dioxus. Windows l'ouvre par open_verified→verify_package_with_trust→validation complète→Wasmtime. Pavé, action multiply=1200, erreur DIVIDE_BY_ZERO, fermeture/réouverture et annulation sont vérifiés. Les hôtes inspectent le même module et le même ZIP ; aucune branche par nom d'outil. Les trois captures package8-* ont été inspectées visuellement ; le résultat et l'erreur sont visibles, sans contrôle tronqué.

La sécurité couvre 10 000 mutations sans panic/instanciation, corpus hostile dirigé, six archives correctement signées mais malformées refusées aux deux frontières, mutations cryptographiques et politiques de clés invalides. Les 48 mutations structurellement valides ne sont pas présentées comme des refus : elles ne peuvent réutiliser la preuve initiale. Le Web refuse les six cas signés sans créer de Worker. Les vérifications sont statiques avant exécution ; les essais de calcul utilisent ensuite le vrai guest.

Validation finale : 98 tests Rust réussis, 1 test HTTPS externe opt-in ignoré ; 44 tests Playwright Chromium/WebKit réussis ; Windows UI réel réussi ; pipeline/reproductibilité et référence gelée réussis. Les cinq schémas antérieurs, les 13 échanges golden et la fixture TOOL1_V1 conservent leurs empreintes. Voir validation-8.3-9.0.json pour commandes, versions, extraits, résultats et hashes des logs.

Le gel porte sur les octets producteurs canoniques, limites d'admission, descripteur, texte signé, chaîne de clés et immutabilité identité/version définis dans CONTRACTS.md. Les lecteurs gardent la compatibilité avec le sous-ensemble historique sûr ; dates/compression historiques ne deviennent pas une exception par outil. Tout changement incompatible exige une version et migration explicites.

Limites maintenues : Linux/macOS et Firefox exclus par le propriétaire ; aucun compte cloud provisionné, publication R2, rotation réseau, compteur minimum durable ou ancre de production. Les clés TEST ne signent aucune livraison réelle. Le contrôle Clippy élargi strict reste non vert à cause d'un avertissement préexistant kortex-ui ; les trois crates de distribution passent en strict et les hôtes passent sans dépendances avec les huit avertissements Desktop déjà présents. Les tests fonctionnels requis passent. Aucun constat ne vaut audit tiers ni validation finale Kortex V1.
