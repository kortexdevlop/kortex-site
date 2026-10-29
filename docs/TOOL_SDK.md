# Créer un outil Core WASM v1

Référence protocolaire : ABI 1.0 et UI 1.0. Le même module est exécuté dans un Worker Web et dans Wasmtime Windows. Aucun identifiant d’outil n’est codé dans les hôtes. Le packager déterministe, les signatures et la publication cloud sont implémentés ; la promotion de production reste un gate distinct.

Pour démarrer, suivre [DEVELOPER_SETUP](DEVELOPER_SETUP.md), puis [TOOL_AUTHOR_GUIDE](TOOL_AUTHOR_GUIDE.md), qui construit le sixième outil réel Base64 depuis un clone propre. Pour déployer : [DEPLOYMENT](DEPLOYMENT.md). Pour les permissions et parcours côté utilisateur : [USER_GUIDE](USER_GUIDE.md). Les recettes de calculatrice ci-dessous servent aussi à comprendre les fixtures historiques et ne remplacent pas ces points d’entrée courants.

## Déclarer et implémenter

Créer une crate sous tools/, l’ajouter au workspace, utiliser `crate-type = ["rlib", "cdylib"]` et dépendre de crates/kortex-sdk ainsi que de serde_json. Reprendre les schémas figés de shared/schemas pour manifest.json et contract.json ; ne pas les étendre pour un outil particulier. Déclarer exactement les actions, formes d’entrée/sortie et permissions nécessaires. Une opération pure n’a aucune permission.

Implémenter `kortex_sdk::Tool` sur un type `Default` et appeler `kortex_sdk::export_tool!(MonOutil)` à la racine de la crate. L’export fournit allocation, libération et dispatch ; ne pas réimplémenter les pointeurs à la main. Les sources de tools/calculatrice montrent une implémentation complète.

Le handler reçoit une Request v1 corrélée par request_id. Il renvoie toujours la même identité de requête avec `success(id, object)` ou `failure(id, code)`. Traiter init (nouvel état), render (racine UI sous forme d’objet), ui.event (état local), action.invoke (action_id et input), puis shutdown (libération de l’état). Les événements UI reconnus sont acquittés ; une erreur de saisie se montre dans un alert. Les mêmes fonctions métier servent aux boutons et aux actions. Une erreur métier d’action reste une Reply structurée, par exemple DIVIDE_BY_ZERO ; elle ne doit pas arrêter le moteur.

L’UI utilise uniquement les types et props du schéma UI v1 : column, row, grid, input, button, alert, etc. Chaque nœud possède type, key stable et unique, props et children. Aucun HTML, CSS ou script guest. Le renderer hôte fournit les labels, le focus, les annonces et les cibles de boutons d’au moins 44 px. Les valeurs input sont des chaînes, même pour une saisie numérique. Le protocole ne transmet pas de keydown : les boutons natifs restent activables avec Entrée/Espace ; les raccourcis globaux spécifiques demanderaient une évolution de contrat.

Le SDK ne fournit pas WASI. Les capacités passent par Host et host_enqueue avec consentement hôte et résultat asynchrone capability.result. Ne jamais appeler réseau, fichiers, processus ou stockage directement depuis un guest. La calculatrice ne lie aucun import.

## Construire et vérifier

Rust est figé à 1.99.1 dans ce dépôt. Installer la cible wasm32-unknown-unknown. La boucle générique actuelle est `xtask build → validate → package`, puis préparation signée de test ; voir le guide auteur. Exemple historique PowerShell de la fixture calculatrice, conservé pour sa référence binaire :

```powershell
$env:RUSTFLAGS='-C link-arg=--max-memory=67108864'
cargo build -p tool-calculatrice --target wasm32-unknown-unknown --release --locked
Remove-Item Env:RUSTFLAGS
node tests/fixtures/calculatrice.cjs
cargo test -p tool-calculatrice --locked
node --test tests/fixtures/calculatrice.test.cjs
cargo test --workspace --locked
pnpm --filter @kortex/web-adapter test
pnpm exec playwright test tests/e2e/calculatrice-web.spec.ts
& tests/e2e/calculatrice-desktop.ps1
```

CARGO_TARGET_DIR est facultatif pour déplacer le cache. Le script tests/fixtures/prepare-calculatrice.mjs construit le guest puis prépare sa fixture ; l’adaptateur Web et le scénario Windows l’appellent. Pour les tests Rust seuls, le lancer avant cargo test. Le shell Web Dioxus doit avoir été construit et servi par le harness de tests/web ; sa copie de test est target/web-dist. Voir les rapports 5.x pour sa construction avec dx. Le scénario Windows exige Windows/WebView2 ; aucun résultat Linux/macOS ou Firefox n’est revendiqué.

Le script générique tests/fixtures/make-tool.cjs accepte manifestPath, contractPath, wasmPath et iconPath. Son enveloppe de test contient ces fichiers et module.wasm dans un ZIP, avec descripteur SHA-256 et signature Ed25519 B.2 séparée. La clé privée éphémère reste en mémoire ; seule la clé publique est écrite sous target/. Aucun mode unsigned ni clé de production. Le même fichier signé sert aux deux hôtes. La fixture est conservée tant que ses octets correspondent et que sa clé n’a pas expiré. Ce helper de régression ne remplace pas le packager de publication disponible via xtask.

## Référence de compatibilité

`tests/golden/calculatrice.ndjson` contient 13 paires Request/Reply : initialisation, rendu, quatre actions, deux erreurs, saisies/clic, rendu après interaction et arrêt. Les tests SDK, moteur natif et Worker Web les exécutent réellement et comparent les objets JSON, sans dépendre de l’ordre des propriétés. Les valeurs numériques et les champs optionnels restent ceux du contrat.

`tests/golden/calculatrice-reference.json` consigne les empreintes du guest, de l’archive et du corpus. Le test Node vérifie aussi la signature du paquet. Une dérive de binaire échoue volontairement contre cette référence : examiner le compilateur, les chemins incorporés et le gate avant de l’actualiser. Les empreintes ne prouvent pas à elles seules une construction reproductible sur tous les OS. Ne jamais régénérer les attentes automatiquement pendant les tests.

Pour un nouvel outil, suivre la même chaîne manifeste/contrat → SDK → WASM → signature → admission vérifiée → tests UI/actions sur les hôtes. Ajouter ses corpus métier et vérifier qu’aucune branche du runtime ne dépend de son nom.

## Pipeline générique disponible depuis 8.2

Les commandes de construction, validation et packaging ne nécessitent plus le helper de fixture propre à la calculatrice : `cargo run -p kortex-xtask --locked -- build --tool <id>`, puis validate/package. --all, --out, --json et --help sont documentés dans SIGNING.md, ainsi que sign/verify et la racine hors ligne. Le helper de phase 7 reste uniquement pour sa référence binaire immuable. Le packager canonique de publication possède une empreinte différente sans changer les échanges du guest ; son format est gelé depuis le gate 8.5, référencé dans CONTRACTS.md.

## Exemples plus complets : les cinq outils

Les cinq outils de la v1 couvrent les modèles à connaître. `tools/calculatrice` : actions pures et interface qui appellent la même fonction. `tools/convertisseur-casse` et `tools/timestamp-utc` : entrées réactives (`input`, `change`) qui recalculent un résultat partagé avec l’action. `tools/calculateur-hash` : cycle `Host::enqueue` + `on_result` pour `file.pick` puis `file.read`, génération qui invalide les résultats tardifs, annulation et refus affichés dans l’interface. `tools/explorateur-csv` : le même cycle avec `file.save` et `storage.*`, plus un traitement en flux.

Règles apprises en les écrivant :
- Un appel guest dispose de 100 millions d’instructions Wasm. Tout traitement proportionnel à un fichier doit être découpé par bloc de lecture et accumuler son état ; ne jamais conserver le fichier entier pour l’analyser au dernier bloc.
- Éviter une allocation par élément (arène + offsets) et les hachages par cellule pour rester dans ce budget.
- Les événements `select` d’un `tabs` portent la clé de l’onglet ; l’outil la mémorise pour le prochain rendu (`active_key`).
- Un état d’erreur doit effacer les résultats précédents : un refus de permission ou un fichier invalide ne laisse aucune table affichée.
- Les capacités sensibles ne se demandent que sur un clic. Un outil sans permission ne déclenche jamais de dialogue, ce que `tests/integration/five-tools.spec.ts` vérifie.
- Écrire les vecteurs partagés dans `tests/integration/five-tools-vectors.json` : le test Web et le test Wasmtime Windows les exécutent à l’identique.
