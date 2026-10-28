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

## Cas des cinq outils

Les cinq outils suivent la même chaîne `build → validate → package → sign → verify → publish` sans option propre à un outil. `scripts/prepare-phase12.mjs` prépare et signe les quatre outils de la phase 12 (graines en mémoire seulement) ; la calculatrice garde sa release `target/staging/calculatrice`, dont le SHA correspond à la version publiée. `scripts/publish-phase12-staging.mjs` accepte `KORTEX_PUBLISH_TOOLS` (liste séparée par des virgules) et `KORTEX_PUBLISH_PROOF` pour ne publier qu’une partie des outils et écrire une preuve distincte.

Si la publication échoue avant écriture, le script restaure la politique d’administration temporaire et consigne l’erreur dans la preuve ; elle peut être relancée telle quelle, car les ressources existantes sont comparées par SHA et jamais écrasées. Une limite de quota Cloudflare (par exemple les lectures D1 quotidiennes du plan gratuit) est un blocage externe : attendre la réinitialisation ou changer de plan relève du propriétaire du compte.

## Vérification finale des cinq outils (28/10/2026)

Les cinq paquets sont maintenant publiés sur le staging, y compris explorateur-csv 1.0.0. Le quota D1 initial est un incident historique résolu ; voir phase12-staging.json. Le test tests/tools/phase12-csv-staging.spec.ts ouvre les cinq fiches depuis le vrai site et compare les neuf vecteurs partagés avec Wasmtime. Le rapport d’export est téléchargé après consentement et vérifié contre export_report.

Le site autorise file.save via le sandbox documenté dans CONTRACTS.md ; aucune republication des paquets n’est nécessaire pour corriger la politique du cadre hôte. Le stockage récent repose sur le véritable IndexedDB et reste refusé lorsque le navigateur ne confirme pas la persistance. Le test positif accorde persistent-storage au navigateur d’un profil temporaire isolé, jamais au profil de l’utilisateur, puis vérifie la relecture après rechargement. Commande : KORTEX_CHROMIUM_CHANNEL=chrome puis pnpm exec playwright test --config tests/tools/playwright.csv-staging.config.ts.
