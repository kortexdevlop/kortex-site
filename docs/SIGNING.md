# Signature des paquets — état 8.2

Date : 27 octobre 2026. Bibliothèques kortex-pack/kortex-verify et CLI kortex-xtask. Ce lot livre une chaîne locale vérifiée ; aucune clé de production, aucun service de publication ni accès R2 n’est configuré. Le format de paquet et le nouveau schéma de descripteur restent candidats jusqu’au gate 8.5.

## Construire et inspecter

```powershell
cargo run -p kortex-xtask --locked -- build --tool calculatrice --json
cargo run -p kortex-xtask --locked -- validate --tool calculatrice --json
cargo run -p kortex-xtask --locked -- package --tool calculatrice --json
cargo run -p kortex-xtask --locked -- package --all --out target/packages-second --json
cargo run -p kortex-xtask --locked -- verify --artifact tests/fixtures/artifact-signed-test.json --json
```

Le scanner cherche tools/*/manifest.json, valide les identifiants et refuse les doublons. La cible cdylib et le chemin WASM viennent de Cargo metadata ; ni le dossier, ni le nom de crate ne sont déduits d’un identifiant d’outil particulier. Build utilise wasm32-unknown-unknown, release, --locked et une mémoire maximale de 64 Mio. Compiler des sources Rust suppose un dépôt source de confiance, comme tout cargo build.

Le ZIP canonique classe les noms lexicalement, utilise DOS 1980-01-01 00:00:00, des permissions Unix 0644 et la méthode stored (octets sans compression). Les objets JSON sont triés récursivement, sans espaces insignifiants ; les tableaux et les valeurs restent inchangés. La reproductibilité est vérifiée à sources/binaire/chaîne identiques, pas revendiquée entre compilateurs ou OS non essayés.

Budget de publication : ZIP ≤25 Mio, somme décompressée ≤25 Mio, entrée/WASM ≤20 Mio, ≤256 fichiers, ratio par entrée ≤20:1. Le lecteur accepte stored et deflate borné, vérifie CRC et cohérence local/central avant extraction en mémoire. Il refuse liens, types spéciaux, collisions de casse, chemins dangereux ou réservés Windows, chiffrement, data descriptors, ZIP64, volumes multiples, extras/commentaires et données cachées hors entrées. Ces limites ne constituent pas une mesure maximale de mémoire du processus.

Les quatre fichiers actuels sont manifest.json, contract.json, module.wasm et l’icône référencée sous assets/. Tout autre fichier dans le paquet est refusé ; les fichiers source Cargo/design/tests ne sont pas emballés. L’asset doit être présent, non vide et borné ; il est traité comme octets, sans prétendre assainir du SVG ou le rendre comme HTML. Le WASM binaire est validé statiquement, avec signatures exactes des exports, mémoire unique 32 bits non partagée plafonnée à 1024 pages, seul import facultatif kortex_v1.host_enqueue et aucun start. Le validateur ne garantit pas la terminaison ni la justesse métier du guest : les tests d’exécution restent nécessaires.

--all concerne build/validate/package ; --out est un répertoire pour build/package, un fichier pour sign. package reconstruit avant validation ; validate inspecte le module déjà construit. Les sorties générées vont sous target/packages par défaut, avec rapport .validation.json. Un paquet existant de mêmes octets est réutilisé ; des octets différents ne remplacent pas silencieusement la même sortie. --json produit le résultat sur stdout et l’erreur structurée sur stderr avec code de sortie non nul. --help décrit les commandes.

## Signer en CI

La graine Ed25519 de 32 octets encodée en base64 standard est injectée exclusivement dans le processus de signature par le gestionnaire de secrets CI, sous KORTEX_SIGNING_KEY_SEED_BASE64, avec KORTEX_SIGNING_KEY_ID public. Elle n’est pas placée dans .env, le navigateur, un argument de commande, un log ou le ZIP. Le contrôle d’environnement applicatif refuse cette variable. Les buffers temporaires du CLI utilisent Zeroizing. Le garde CI=true est une protection contre l’usage accidentel, pas une frontière de sécurité contre un opérateur contrôlant son environnement.

Dans un job autorisé, avec ces secrets déjà injectés :

```text
cargo run -p kortex-xtask --locked -- sign --artifact target/packages/calculatrice-1.0.0.tool --json
```

La sortie détachée est calculatrice-1.0.0.tool.json, sauf --out explicite. Le CLI vérifie d’abord le paquet et tire id/version/ABI du manifeste, puis signe le texte UTF-8 B.2 exactement :

```text
kortex-artifact-v1\n{tool_id}\n{version}\n{abi_version}\n{sha256_hex_lower}\n
```

Chaque \n représente un octet LF ; le dernier LF est obligatoire. La signature porte sur ce texte, avec le SHA-256 du ZIP final, jamais sur un digest de descripteur ni sur un ZIP contenant sa signature. Le descripteur comporte exactement schema_version=1, tool_id, version, abi_version, sha256, signing_key_id et signature base64 canonique. Le nouveau schéma est partagé Rust/AJV, avec validateur Web précompilé compatible CSP.

## Vérifier et renouveler les clés

verify contrôle le hash avant la signature de l’artefact et avant sa décompression, puis valide les métadonnées et l’ABI. --archive permet de préciser le ZIP ; sinon son nom est inféré du descripteur voisin. Pour les vrais environnements, fournir explicitement --root et --trust à partir de fichiers de confiance de l’hôte. Les valeurs par défaut pointent uniquement vers les fixtures publiques TEST du dépôt ; le résultat le signale par trust_profile=test-only-default. Les clés TEST ne doivent pas être configurées comme confiance de production.

RootKey contient id et public_key_base64. SignedKeys contient schema_version=1, version croissante, issued_at/expires en millisecondes UNIX, root_key_id, keys et signature. Chaque clé contient id, public_key_base64, not_before, expires et revoked. Les noms sont ASCII bornés, les entiers dans la plage sûre JavaScript, les encodages base64 sont canoniques et les doublons refusés. La racine publique est épinglée par l’hôte, jamais choisie dans le paquet.

Le texte signé du manifeste de clés commence par `kortex-keys-v1\n{version}\n{issued_at}\n{expires}\n{root_key_id}\n`, suivi des clés triées par identifiant ASCII, une ligne `id|public_key_base64|not_before|expires|revoked\n` par clé ; booléens true/false minuscules. Les noms interdisent les séparateurs et sauts de ligne. Ce format de snapshot est versionné et candidat ; Rust et Web vérifient la même fixture publique.

Pour une rotation : générer la nouvelle clé de publication dans le gestionnaire de secrets, confier seulement sa partie publique à l’opérateur hors ligne, incrémenter la version de SignedKeys, ajuster les fenêtres, puis signer son texte avec la racine hors ligne. L’hôte vérifie la racine, les dates et sa version minimale déjà acceptée avant d’utiliser les nouvelles clés. Une clé supprimée ou revoked=true ne vérifie plus d’artefact. Le CLI expose --min-trust-version ; il ne persiste pas automatiquement ce compteur. La distribution réseau et le stockage durable de la politique de révocation relèvent des étapes suivantes, pas d’une garantie déjà déployée.

En cas de compromission d’une clé de publication : suspendre les signatures, créer hors ligne un snapshot supérieur qui la révoque et autorise une nouvelle clé, puis le distribuer par le canal hôte authentifié prévu. Une racine compromise exige un remplacement explicite de l’ancre hôte par un canal de confiance indépendant ; un fichier fourni avec un paquet ne peut pas s’autoriser lui-même. Les procédures d’urgence et de révocation distante complètes sont prévues plus tard dans le plan.

## Fixtures et contrôles

scripts/generate-test-signing-key.mjs génère une racine TEST et une clé de publication TEST en mémoire. Il ne les exporte jamais comme clés privées et n’écrit que sous target/. Exemple :

```text
node scripts/generate-test-signing-key.mjs --archive target/packages/calculatrice-1.0.0.tool --manifest tools/calculatrice/manifest.json
```

Le dépôt conserve seulement le paquet calculatrice de test, son descripteur signé et les deux documents publics root/keys. Leur signature ne dépend d’aucun secret disponible pour les tests. Les graines déterministes des tests unitaires sont des vecteurs TEST volontairement publics et ne correspondent pas aux ancres de production. Aucun de ces matériaux ne crée une identité de publication réelle.

Tests : cargo test -p kortex-pack --locked ; cargo test -p kortex-verify --locked ; cargo test -p kortex-xtask --locked ; pnpm --filter @kortex/web-adapter test. Les tests couvrent déterminisme, ZIP hostile, discovery arbitraire, injection CI simulée en mémoire, champs modifiés, signature, clé retirée/révoquée/expirée, snapshot falsifié et version minimale. Le même paquet calculatrice canonique produit 1200 dans les deux navigateurs. La référence TOOL1_V1 de phase 7 reste inchangée ; la compatibilité complète et le gel de paquet phase 8 restent à 8.3–8.5.
