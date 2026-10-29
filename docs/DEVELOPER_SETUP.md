# Démarrer le développement sous Windows

Ce guide vise Windows x64. Le navigateur testé est Chromium ; le Desktop utilise WebView2. Linux, macOS, Firefox et WebKit ne font pas partie du périmètre demandé. Les étapes locales ci-dessous ne nécessitent ni compte Cloudflare, ni clé privée, ni compte utilisateur Kortex.

## Prérequis

Installer Git, Node **24.15.0**, Rust via rustup et Visual Studio Build Tools 2022 avec la charge **Développement Desktop en C++** et le SDK Windows. Le compilateur hôte Rust est MSVC. Ouvrir un nouveau PowerShell après l’installation afin de récupérer le PATH.

Les versions viennent de `rust-toolchain.toml`, `package.json`, des lockfiles et de `.github/workflows/validate.yml`, pas d’une commande installant automatiquement les dernières versions. Le tableau de [baseline](DEPENDENCY_BASELINE.md) contient aussi des notes historiques ; les fichiers verrouillés du dépôt font foi.

```powershell
git --version
node --version
npm install --global pnpm@11.25.0
pnpm --version
rustup toolchain install 1.99.1 --profile minimal --component rustfmt --component clippy --target wasm32-unknown-unknown
```

Si `pnpm` reste introuvable, rouvrir PowerShell et vérifier `Get-Command pnpm.cmd`. Sur Windows, `pnpm.cmd` permet aussi d’éviter le blocage du lanceur PowerShell sans changer la politique de sécurité du système.

## Obtenir les sources et construire un premier outil

Les guides 21.1 et les nouveaux gates se trouvent actuellement dans la branche de validation de la [PR 15](https://github.com/kortexdevlop/kortex-site/pull/15), pas encore dans `main`. Ce choix est explicite pour rendre les commandes suivantes reproductibles.

```powershell
git clone --branch codex/runtime-performance-accessibility https://github.com/kortexdevlop/kortex-site.git Kortex
cd Kortex
pnpm.cmd install --frozen-lockfile
$env:CARGO_INCREMENTAL='0'
$env:CARGO_PROFILE_DEV_DEBUG='0'
cargo fetch --locked
cargo run -p kortex-xtask --locked -- build --tool encodeur-base64 --json
cargo run -p kortex-xtask --locked -- validate --tool encodeur-base64 --json
cargo run -p kortex-xtask --locked -- package --tool encodeur-base64 --out target/onboarding-packages --json
cargo test -p tool-encodeur-base64 --locked
```

Chaque commande doit réussir avant la suivante. Le sixième outil est un exemple réel, sans permission sensible. `build` produit le module `wasm32-unknown-unknown`, `validate` contrôle manifeste/contrat/WASM et `package` crée son archive déterministe et son rapport sous `target/onboarding-packages`. Cette archive n’est pas encore une publication approuvée.

Après ces commandes, une préparation signée **de test uniquement** est possible :

```powershell
node scripts/release-tool.mjs --tool encodeur-base64 --dry-run --out target/onboarding-base64
```

Le script reconstruit, valide, emballe, signe et vérifie avec des clés éphémères. Il écrit `release-report.json`, `publication-request.json`, les documents publics de confiance et le paquet sous `target/onboarding-base64`, sans upload. Ne pas placer ce paquet ou sa racine de test dans le canal de production. Changer de dossier `--out` si la version ou les octets ont changé : les reprises ne permettent pas d’écraser une version immuable.

La recette a été exécutée sur une copie propre du dépôt ; les résultats et les limites de cet essai figurent dans [STEP-21.1](reports/STEP-21.1.md).

## Préparer l’interface complète, facultatif pour écrire un outil pur

Installer Dioxus CLI **0.7.10**. Le téléchargement et le contrôle ci-dessous reprennent le workflow réellement utilisé :

```powershell
$taskDxDir=Join-Path (Get-Location) 'target/onboarding-tools/dx'
$taskDxZip=Join-Path (Get-Location) 'target/onboarding-tools/dx.zip'
New-Item -ItemType Directory -Force (Split-Path $taskDxZip) | Out-Null
Invoke-WebRequest 'https://github.com/DioxusLabs/dioxus/releases/download/v0.7.10/dx-x86_64-pc-windows-msvc.zip' -OutFile $taskDxZip
if ((Get-FileHash -LiteralPath $taskDxZip -Algorithm SHA256).Hash.ToLowerInvariant() -ne '45eb4f87b7f86fdba8508ee7e0ef1f9e13cf8f2ffcc6a2d5858d0e8ea7dd1531') { throw 'Dioxus checksum mismatch' }
Expand-Archive -LiteralPath $taskDxZip -DestinationPath $taskDxDir -Force
$env:PATH="$taskDxDir;$env:PATH"
pnpm.cmd exec playwright install chromium
node scripts/ci-prepare.mjs
node website/node_modules/astro/bin/astro.mjs build --root website
node scripts/qa-local-system.mjs
```

Attendre `QA_LOCAL_SYSTEM_READY`. Ouvrir `http://127.0.0.1:4322/` ou `/workspace`. L’API locale est sur `4332`, le Runtime sur `4342`. Les cinq outils du système QA sont réellement signés, stockés et admis par les composants locaux ; ce lanceur n’utilise aucun binding distant. Leurs clés expirent après une heure : arrêter puis relancer le système pour obtenir un nouvel environnement. Arrêter avec Ctrl+C ; ne pas arrêter un service tiers occupant déjà ces ports.

Pour les seuls composants visuels de développement : `node website/node_modules/astro/bin/astro.mjs dev --root website --host 127.0.0.1 --port 4321`, puis `http://127.0.0.1:4321/__dev/design`. Cette démonstration n’est pas le catalogue réel et ne publie pas ses cartes.

## Choisir les contrôles utiles

| Travail | Commande depuis la racine |
|---|---|
| Un outil | `cargo test -p tool-encodeur-base64 --locked` puis `xtask build/validate/package` ci-dessus |
| Contrats et règles de publication | `node scripts/ci-verify.mjs --lane contracts` |
| API locale | `pnpm.cmd --filter @kortex/api test` |
| Tous les composants Rust | `node scripts/ci-prepare.mjs --native`, puis les commandes ci-dessous |
| Interface, performance, résilience locales | `node scripts/qa-verify.mjs` après les builds ci-dessus, avec les ports 4321–4342 libres |
| Vérification CI complète | `node scripts/ci-verify.mjs` ; comprend des tests HTTPS de staging et des audits réseau |

Pour la suite Rust, utiliser les nouvelles fixtures plutôt que d’anciens dossiers de campagnes :

```powershell
$env:KORTEX_TEST_RELEASES=Join-Path (Get-Location) 'target/ci-fixtures'
cargo test --workspace --locked
```

Le smoke HTTPS facultatif ignoré par défaut a été exécuté séparément au gate 20.4 ; les commandes et preuves sont dans [la recette native](../tests/resilience/native.md). Un échec distant D1 ne signifie pas que le système local dépend de Cloudflare. Ne pas rejouer le test catalogue 10k sur le compte servant le produit : voir [l’incident D1](reports/D1_QUOTA_INCIDENT.md).

## Résoudre un échec courant

| Symptôme | Vérification |
|---|---|
| `link.exe` absent | Build Tools C++/SDK Windows installés ; essayer le Developer PowerShell de Visual Studio |
| Cible WASM absente | `rustup target add wasm32-unknown-unknown --toolchain 1.99.1` |
| `--locked` refuse un changement | Vérifier le diff du lockfile ; ne pas supprimer le lock pour contourner le contrôle |
| Signer non construit | Exécuter d’abord une commande `cargo run -p kortex-xtask` ; le dry-run appelle ce binaire compilé |
| Paquet refusé ou confiance expirée | Recréer les fixtures de test ; ne pas désactiver signatures/révocations |
| Port local occupé | Arrêter uniquement son ancien serveur QA ou choisir un environnement de travail libre |
| Build très volumineux | Prévoir plusieurs Go libres ; le cache `target` est régénérable, les sources et données utilisateur ne le sont pas |

Pour modifier un outil ou en ajouter un : [guide auteur](TOOL_AUTHOR_GUIDE.md). Pour Cloudflare et les secrets opérateur : [déploiement](DEPLOYMENT.md).
