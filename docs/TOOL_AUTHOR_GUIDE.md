# Écrire et livrer un outil

Le sixième outil réel, `tools/encodeur-base64`, démontre l’ajout générique sans modifier l’API, le Runtime ou le renderer pour son nom. Commencer par sa construction et son test dans [DEVELOPER_SETUP](DEVELOPER_SETUP.md). Le détail des messages et composants se trouve dans [TOOL_SDK](TOOL_SDK.md) et les contrats figés dans [CONTRACTS](CONTRACTS.md).

## Structure et identité

Un outil sous `tools/<id>/` contient `Cargo.toml`, `src/lib.rs`, `manifest.json`, `contract.json` et l’icône désignée sous `assets/`. Le workspace couvre déjà `tools/*` ; ne pas ajouter une exception par outil dans les hôtes. Utiliser un identifiant slug unique et cohérent entre `manifest.id` et `contract.tool_id`. Le nom de crate reste distinct, par convention `tool-<id>`.

Pour créer son propre dossier, reprendre la structure de Base64, puis changer ces trois identités, le nom/description/catégorie, l’icône et les actions. Mettre à jour le lockfile après ajout de la crate : un premier `cargo check -p tool-mon-outil --offline` peut l’actualiser si les dépendances nécessaires sont déjà disponibles. Examiner le diff de `Cargo.lock` avant les commandes suivantes en `--locked`. Une dépendance nouvelle nécessite une résolution et une revue explicites, pas un changement silencieux de la chaîne verrouillée.

La crate exporte `rlib` et `cdylib`, utilise `kortex-sdk`, implémente `Tool` sur un état `Default` puis appelle `kortex_sdk::export_tool!(MonOutil)`. Garder la même fonction métier derrière le bouton UI et `action.invoke`. Gérer `init`, `render`, `ui.event`, `action.invoke` et `shutdown` avec une réponse corrélée au `request_id`.

## Contrats, UI et permissions

Déclarer les schémas d’entrée/sortie, délais, effet de bord, idempotence et permissions dans le contrat. Une action pure, comme l’encodage Base64, déclare zéro permission. Respecter les versions ABI/UI et `min_runtime` ; ne pas étendre les schémas pour contourner un refus.

Le guest transmet un arbre UI déclaratif : clés stables, composants et props autorisés, labels et messages d’erreur. Aucun HTML/CSS/JavaScript guest, import WASI ou accès direct au système. Les fichiers, réseau et stockage passent par les capacités hôte ; l’utilisateur peut refuser et les droits expirent ou sont révoqués. Découper les traitements volumineux en blocs pour respecter mémoire et fuel. Effacer un résultat périmé après erreur ou refus.

Références utiles : calculatrice pour les actions pures ; convertisseur-casse/timestamp-utc pour les champs réactifs ; calculateur-hash pour lecture locale et annulation ; explorateur-csv pour lecture en flux, export et stockage autorisé. Les résultats ne sont pas automatiquement renvoyés à l’IA.

## Boucle locale de référence

Ces commandes portent sur le sixième outil existant et ont été exécutées. Pour sa propre crate, remplacer les identifiants après les adaptations décrites plus haut :

```powershell
cargo test -p tool-encodeur-base64 --locked
cargo run -p kortex-xtask --locked -- build --tool encodeur-base64 --json
cargo run -p kortex-xtask --locked -- validate --tool encodeur-base64 --json
cargo run -p kortex-xtask --locked -- package --tool encodeur-base64 --out target/onboarding-packages --json
node scripts/release-tool.mjs --tool encodeur-base64 --dry-run --out target/onboarding-base64
```

`xtask` découvre manifeste et crate via les métadonnées Cargo. Le packager produit le ZIP canonique ; le dry-run signe et vérifie avec une racine éphémère et ne publie rien. Les sources ne sont pas incluses dans le ZIP : manifeste, contrat, module et icône constituent le paquet admis. Limites et format exact : [SIGNING](SIGNING.md).

Écrire des tests métier positifs/négatifs et un corpus partagé d’actions. Vérifier le même paquet signé dans Worker Web et Wasmtime, les erreurs, les permissions refusées et l’annulation si l’outil demande un accès. Le corpus des cinq outils de référence est `tests/integration/five-tools-vectors.json`. Base64 possède ses propres tests ; il n’est pas ajouté artificiellement à un gate qui exige exactement les cinq outils V1.

## Signer et publier

La signature Ed25519 de l’archive, la signature des documents de confiance et la signature Authenticode de l’installateur Windows ont des rôles distincts. Le développeur n’a besoin d’aucune clé de production pour tester. La racine privée reste hors CI ; la clé déléguée est fournie au job de signature protégé, pas à Cargo ni à une PR.

Après revue et validation, le workflow `.github/workflows/tools.yml` teste/construit/valide/emballe les outils modifiés sur `main`, puis publie si l’environnement protégé et `KORTEX_STAGING_PUBLISH_ENABLED` l’autorisent. Le job impose une CI réussie sur le même commit, vérifie les archives et la provenance, puis publie via R2 et l’API/D1. Son déclenchement manuel accepte aussi le champ `tool`.

Ne pas republier une version existante avec d’autres octets. Pour une correction, augmenter la version ; pour une coupure de publication, reprendre le même artefact et la même requête idempotente. Pour retirer ou revenir en arrière, utiliser un plan de maintenance relu : [DEPLOYMENT](DEPLOYMENT.md) et [plans](../infra/releases/README.md). Un retour du pointeur de catalogue n’annule pas une révocation connue des clients.

Le catalogue dynamique reçoit la nouvelle fiche sans ajouter de branche conditionnelle dans le site ou les hôtes. Le déploiement de production et la validation V1 restent des gates séparés ; créer un outil n’accorde pas l’autorisation de promouvoir la plateforme.
