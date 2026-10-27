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

## Explorateur CSV

Contrat : `summarize_csv({data_text})` et `export_report({data_text})`, deux actions pures sans permission. `data_text` est limité à 1 000 000 d’octets UTF-8 (l’enveloppe ABI reste à 1 Mio). `summarize_csv` renvoie `{rows,columns,stats[]}` ; chaque entrée contient `name`, `kind` (`number` ou `text`), `count`, `nulls` et, pour les colonnes numériques seulement, `min`, `max`, `mean`. `export_report` renvoie `{report_csv}`. L’interface et les actions passent par le même analyseur.

Parseur strict, sans crate externe : UTF-8 obligatoire (BOM accepté au début), virgule comme séparateur, `LF` ou `CRLF`, guillemets seulement autour d’un champ complet (`""` pour un guillemet), en-tête obligatoire, mêmes colonnes sur chaque ligne. Refus explicites : `INVALID_UTF8`, `MALFORMED_QUOTE`, `UNTERMINATED_QUOTE`, `RAGGED_ROW`, `INVALID_LINE_ENDING` (CR seul), `INVALID_CHARACTER`, `EMPTY`, `TOO_LARGE`. Bornes : 5 000 lignes de données, 50 colonnes, 10 Mio de fichier côté interface, 1 000 000 d’octets côté action. Aucun nombre `NaN`/infini n’est produit : une valeur qui déborde donne `NON_FINITE_NUMBER`, une somme qui déborde `NUMERIC_OVERFLOW`.

Politique de vide : une cellule vide ou faite d’espaces est nulle, comptée dans `nulls`, jamais dans `count`. Une colonne est numérique si toutes ses valeurs non nulles sont des décimaux stricts (`+1.5`, `.5`, `1e3`) ; `0x10`, `1_000`, `NaN` ou `inf` la rendent textuelle. Le zéro négatif est normalisé. Les statistiques « valeurs distinctes » ont été retirées : elles ne tiennent pas dans le budget d’instructions d’un appel.

Le budget par appel guest est de 100 millions d’instructions Wasm (`FUEL`). Le fichier est donc lu par blocs de 256 Kio et analysé au fil de l’eau : chaque bloc complète le parseur en flux (arène unique de cellules, pas d’allocation par cellule) et les accumulateurs de statistiques, de sorte que le dernier appel ne fait que finaliser. Un analyseur à tampon complet dépassait ce budget sur un fichier de 880 Ko.

Interface : onglets Aperçu, Statistiques et Graphique construits uniquement avec `tabs`, `table`, `select`, `input`, `svg-chart` génériques. Le tableau paginé affiche 50 lignes (le schéma accepte 500 lignes et 2 000 nœuds). La recherche porte sur la colonne choisie, sans casse. Le graphique prend une colonne numérique, au plus 500 valeurs non vides numérotées par ligne du fichier, en barres ou en courbe ; les valeurs hors ±2^53 sont refusées avec un message.

Permissions : `file.pick`, `file.read`, `file.save`, `storage.read`, `storage.write`. L’export demande `file.save` à chaque clic ; un refus affiche « Autorisation refusée » sans rien écrire. Les noms de colonnes du rapport sont neutralisés contre l’injection de formules (`=`, `+`, `-`, `@`, tabulation, CR préfixés d’une apostrophe). Les fichiers récents (5 noms au plus, clé `prefs/recent`) sont lus ou écrits seulement sur clic explicite, jamais à l’ouverture. Le stockage Web exige une origine d’exécution dédiée et un stockage persistant confirmé par le navigateur : sans eux, l’outil affiche l’échec fermé (`PERMISSION` ou `UNAVAILABLE`).
