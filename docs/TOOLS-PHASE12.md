# Outils 2 à 4 — règles et limites

Ces trois outils utilisent le SDK, le contrat d’actions, le renderer et le format signé existants. Aucun accès réseau guest ; aucune donnée de fichier envoyée au cloud. Publication limitée au staging, indépendante du build du site.

## Convertisseur de casse

Actions `upper`, `lower`, `title`, entrée `{text}`, sortie `{text}`. Limite : 32 768 octets UTF-8, vérifiée dans le guest en plus de la limite de chaîne du schéma. Majuscules/minuscules utilisent les tables Unicode de Rust ; les expansions comme ß→SS sont conservées. Aucune normalisation NFC/NFD implicite : une séquence décomposée reste décomposée.

`title` met le texte en minuscules, puis capitalise le premier caractère alphabétique au début et après un espace Unicode ou un tiret. Une apostrophe ne commence pas un nouveau mot : `l’ÉTÉ - 42éLÈVES` devient `L’été - 42Élèves`. Ce n’est pas une promesse de titlecase universel ou de règles linguistiques pour toutes les langues. La saisie réactive et les boutons appellent la même fonction que les actions autonomes.

## Calculateur SHA-256

L’interface demande `file.pick`, puis `file.read`. Les handles sont opaques, limités à l’instance par l’hôte et jamais rendus comme chemins. Taille fichier maximale : 10 Mio ; lecture incrémentale par blocs de 256 Kio au plus. Validation des offsets, taille, EOF et base64 avant mise à jour du hash. Aucun stockage du fichier complet dans le guest.

Refuser la permission ne lit aucun octet. L’annulation efface l’état/résultat, interdit les lectures suivantes et ignore un résultat en retard ; une lecture bornée déjà en cours peut se terminer côté hôte. Le protocole gelé ne contient pas d’appel pour annuler rétroactivement cette opération. Les handles restent soumis à la révocation/expiration de l’hôte.

Action autonome `sha256_bytes({data_base64})->{hex}` : résultat seulement, 64 chiffres hexadécimaux minuscules. Aucune permission de fichier nécessaire pour des octets explicitement fournis à l’action. L’enveloppe ABI totale reste limitée à 1 Mio : le champ base64 est donc plafonné à 1 048 000 caractères, soit au plus 786 000 octets décodés. Un fichier plus grand passe par l’interface incrémentale, et non par un élargissement du contrat ABI. Aucun fichier ni handle n’est renvoyé par défaut à une future orchestration IA.

## Dates UTC et Unix

`to_unix({iso_utc})` et `from_unix({value,unit})` renvoient `{iso_utc,unix_seconds,unix_millis}`. L’unité est toujours explicite : `seconds` ou `milliseconds`. Les nombres doivent être entiers et la date doit rester entre 0001-01-01 et 9999-12-31 inclus.

Les entrées ISO exigent `T` et `Z` majuscules ; pas de fuseau absent ni d’offset local. Pas de lecture de l’horloge système, timezone OS ou DST. La précision inférieure à la milliseconde et les secondes intercalaires sont refusées. La sortie ISO est canonique UTC ; les secondes négatives utilisent la division euclidienne : -1 milliseconde donne `1969-12-31T23:59:59.999Z`, -1 seconde et -1 milliseconde.

## Reproduction

```powershell
cargo test -p tool-convertisseur-casse -p tool-calculateur-hash -p tool-timestamp-utc --locked --offline
node scripts/prepare-phase12.mjs
pnpm exec playwright test --config tests/tools/playwright.config.ts
cargo test -p kortex-runtime-native --test phase12 --locked --offline
cargo build -p kortex-runtime-desktop --locked --offline
pwsh -File tests/tools/phase12-desktop.ps1
node scripts/publish-phase12-staging.mjs
pnpm exec playwright test --config tests/tools/playwright.staging.config.ts
```

Les releases préparées sont immuables sous target/phase12. Le script de préparation exécute le vrai pipeline build/validate/package/sign/verify ; aucune clé privée n’est écrite sur disque. Les ancres et clés publiques signées sont des configurations de confiance de staging valables sept jours, séparées de la production. Le publieur vérifie l’identité des ressources existantes, conserve l’audit et restaure sa politique d’administration temporaire. L’opérateur local simule la CI, sans prétendre à une exécution GitHub OIDC.

Les tests Windows du backend utilisent une sélection de fichier contrôlée à la frontière du picker, puis de vrais handles, permissions, lectures disque et WASM Wasmtime. Le dialogue natif Windows de sélection positive n’est pas automatisé. Les tests Web utilisent le vrai picker navigateur ; les tests WebView2 couvrent aussi l’interface des trois outils et le refus de permission.
