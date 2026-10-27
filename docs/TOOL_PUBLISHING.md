# Préparer une publication

Depuis la racine du dépôt, avec les versions Rust/Node verrouillées et les dépendances déjà en cache :

```text
node scripts/release-tool.mjs --tool calculatrice --dry-run
node --test tests/publication/*.test.mjs
```

Le script générique découvre l'outil via le CLI puis enchaîne build, validation, ZIP canonique, hash SHA-256 indépendant, signature et vérification. Il écrit uniquement sous `target/staging/<id>/` : paquet et descripteur dans packages/, documents publics root.json/keys.json, configuration de confiance hôte host-trust.json, publication-request.json et release-report.json. `--out` permet un autre sous-répertoire de target/ ; les sorties hors de ce répertoire et liens sortants sont refusés. Cargo fonctionne hors ligne. Aucun client HTTP, upload R2 ou commande de déploiement ne fait partie du script.

`--dry-run` construit de vrais octets WASM/ZIP et une vraie signature Ed25519 avec des clés TEST éphémères. Il ne simule pas une signature de production. Les graines ne sont écrites ni en fichier ni dans les logs ; seule la graine de publication TEST est injectée au processus sign, puis son buffer est effacé. Les objets privés Node restent limités à la durée du processus. Les documents publics expirent après 24 h : refaire un essai dans un nouveau `--out` après expiration, sans réécrire une ancienne preuve.

Sans `--dry-run`, la préparation reste locale mais exige `CI=true`, `KORTEX_SIGNING_KEY_SEED_BASE64`, `KORTEX_SIGNING_KEY_ID`, `--root <ancre publique>` et `--trust <snapshot signé>`. La racine doit provenir de la configuration hôte autorisée, jamais du guest. Voir SIGNING.md pour les clés/rotations. Le minimum de version est fourni par l'opérateur ; sa persistance et sa distribution distante ne sont pas déployées ici.

PublicationRequest contient exactement `descriptor`, `manifest`, `contract`, `r2_key`, `request_id`. Le CLI valide les schémas réels du paquet et la preuve ; l'orchestrateur vérifie leur cohérence et la clé `tools/{id}/{version}/{sha256}.tool`. Le UUID est créé une fois ou fourni par `--request-id`. Une répétition dans le même staging conserve ce UUID et la preuve, sans signer de nouveau. Un changement de profil TEST/CI, de contenu ou de version dans ce staging est refusé ; un fichier .tool existant avec des octets différents n'est jamais écrasé. La requête d'exemple dans shared/fixtures utilise uniquement la fixture publique TEST permanente.

Une publication future doit créer une nouvelle version pour changer le contenu. Le rollback doit déplacer transactionnellement le pointeur du catalogue vers une version antérieure encore autorisée, sans réécrire ses octets ni son descripteur. R2, transactions de publication et API sont les étapes suivantes de la phase 9 ; ce script ne les réalise pas. Après un staging incomplet, conserver les éléments pour diagnostic puis utiliser un nouveau --out.

Le gate local admet le paquet par les chemins normaux Web et Windows, avec ancre et snapshot signés fournis à l'hôte. Référence immuable : tests/golden/calculatrice-artifact.json ; critères du format : CONTRACTS.md, PACKAGE_FORMAT_V1.
