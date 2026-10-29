# Déploiement et reprise

Ce guide décrit les scripts et workflows existants, pas une autorisation de production. Le périmètre est Windows x64 / Chromium-WebView2. La RC n’est pas gelée : CI finale, staging après incident D1, artefacts de même révision, signature Windows et approbation restent ouverts. Voir [RELEASE_CANDIDATE](reports/RELEASE_CANDIDATE.md) et [registre](PROGRESS.md).

## Environnements et ressources

| Composant | Configuration faisant foi | Rôle |
|---|---|---|
| API / D1 / R2 | `infra/deploy/channels.json`, `api/wrangler.jsonc` | Binding D1 `DB`, bucket privé `TOOLS_BUCKET`, API d’administration authentifiée |
| Site Astro SSR | `website/wrangler.jsonc` et configuration générée par son build | Bindings de service API, identité et sitemap |
| Runtime Web | `runtime/web/worker.ts`, `target/web-dist`, documents de confiance publics | Origine séparée, iframe et téléchargements vérifiés |
| Outils | Manifestes/contrats et requêtes de publication signées | Archive R2 immuable, métadonnées/état de publication D1 |
| Windows | `desktop/Dioxus.toml`, `scripts/desktop-package.mjs` | Installateur EXE NSIS x64, WebView2, canaux et provenance |

Les IDs réels staging sont déjà inscrits dans ces configurations ; ne pas créer de nouvelles ressources à chaque déploiement. Les valeurs toutes à zéro du système local ne sont pas des IDs Cloudflare réels. `stable`/production restent non configurés ou bloqués par les gates. Conserver la zone DNS et la messagerie existantes ; aucun nettoyage de domaine n’est une étape de ce guide.

Pour un premier compte réellement vierge : inventorier compte/profil, zone et ressources ; créer une D1 et un bucket privés dédiés à l’environnement, puis renseigner leurs IDs et les origines séparées. Configurer les bindings et ancres publiques avant toute publication. Les vérifications sont décrites dans [la checklist staging](../infra/staging-checklist.md) et [la configuration Cloudflare](../infra/cloudflare-setup.md), dont les checkpoints datés sont historiques. Il n’existe pas de commande bootstrap production « tout-en-un » validée : ne pas présenter les commandes locales de QA comme du provisioning distant.

## Identités et secrets

Les PR construisent et testent sans secret de staging. La publication utilise `main`, une CI réussie sur le même SHA et l’environnement GitHub `staging` protégé. Le token Cloudflare reste dans ses secrets protégés ; la clé Ed25519 déléguée est fournie au seul job de signature. La racine privée reste chez l’opérateur, hors GitHub et hors application. Les noms exacts utilisés sont notamment `CLOUDFLARE_API_TOKEN`, `KORTEX_SIGNING_KEY_SEED_BASE64` et la variable publique `KORTEX_SIGNING_KEY_ID` ; consulter le workflow ciblé avant toute configuration.

L’identité OIDC GitHub du publisher est distincte du fournisseur Google des utilisateurs. Le premier identifie un workflow autorisé auprès de l’API ; le second reste désactivé tant que ses clients ne sont pas créés. Aucun token ne doit apparaître dans une URL, un exemple de plan ou un rapport.

Pour Google : préparer les clients Web et Desktop, l’écran de consentement et les utilisateurs de test. Les identifiants publics, issuer, origines d’endpoints autorisées et URI de retour sont décrits dans `infra/identity/google-staging.json` et les variables `OIDC_*` lues par `api/src/services/identity-config.ts`. Le secret Web reste uniquement côté serveur. Le flux utilise code + PKCE S256 ; ne pas copier le secret dans le Desktop. Vérifier discovery/JWKS puis une vraie connexion avant d’activer la fonction. La simple présence d’IDs ou d’une discovery ne prouve pas ce parcours. L’email/mot de passe reste une option distincte ; vérification et récupération email attendent un service d’envoi.

## Construire et publier sur staging

Les préparations locales de [DEVELOPER_SETUP](DEVELOPER_SETUP.md) n’envoient rien. Les commandes de la colonne suivante sont des entrées **de jobs protégés**, pas des remplacements à lancer dans un terminal sans contexte CI :

| Workflow | Action réelle |
|---|---|
| `validate.yml` | `node scripts/ci-verify.mjs` réparti sur contrats, natif, Web et sécurité |
| `api.yml` | `node scripts/ci-deploy.mjs api staging` : bookmark D1, migrations, Worker, smoke |
| `tools.yml` | Détecte/teste/construit/valide/emballe les outils, puis `ci-publish-tool.mjs` publie si l’environnement et `KORTEX_STAGING_PUBLISH_ENABLED` l’autorisent |
| `website.yml` | Build Astro et Runtime ; `ci-deploy.mjs website staging`, puis `ci-deploy.mjs runtime staging`, puis `publish-runtime.mjs` |
| `desktop.yml` | Construit/teste l’EXE, puis publication staging si `KORTEX_STAGING_DEPLOY_ENABLED` l’autorise |

Ordre logique d’une livraison complète : sauvegarde/migration D1, API compatible, archives et catalogue, Runtime, site, puis Windows. Les workflows actuels sont indépendants : celui du site déploie concrètement **site puis Runtime puis canal signé**. Ce regroupement ne constitue pas l’orchestrateur de promotion production de 22.1 ; une évolution incompatible ne doit pas être livrée en supposant un autre ordre. La production reste refusée dans les scripts actuels.

R2 reçoit des archives adressées par hash ; l’API vérifie les octets et signatures avant de publier l’état D1. Réutiliser les mêmes octets et le même `request_id` pour reprendre une coupure. Une version existante ne s’écrase pas. Une mise à jour d’un outil n’impose pas de rebuild du site dynamique. Le Runtime et l’installateur possèdent aussi leurs versions et canaux signés ; le bundle local 1.0.2 préparé n’est pas une preuve de promotion distante.

Le build Windows local est `node scripts/desktop-package.mjs` après installation des prérequis Dioxus/NSIS/WebView2. Il fabrique une préversion staging non signée Authenticode, pas une release de production. La reprise CI `artifact_run` n’est permise que pour un installateur d’origine prouvée dont les sources pertinentes n’ont pas changé. Voir [distribution Windows](DESKTOP_DISTRIBUTION.md).

## Vérifier après chaque bascule

Une fois staging disponible, la commande de lecture seule suivante vérifie les surfaces réelles :

```powershell
node scripts/smoke-cloud.mjs --env staging
```

Elle vérifie health, catalogue/détail, document de révocation, fiche SSR, sitemap, bureau, cadre Runtime et téléchargement/hash. Les jobs peuvent cibler `--target api`, `website` ou `runtime`. Ce smoke n’invente pas une connexion Google ni une signature Windows ; garder leurs tests séparés. Un état 200 de health seul ne valide pas D1 ou un outil.

Après l’incident de charge, attendre la remise à disposition du quota puis effectuer les vrais contrôles ; ne pas remettre les compteurs à zéro, changer de forfait ou rejouer 10k fiches sur le compte produit. Les futurs essais de charge exigent un compte QA distinct et des budgets explicites : [incident D1](reports/D1_QUOTA_INCIDENT.md).

## Rollback, retrait et incident

Si le smoke de déploiement échoue, `ci-deploy.mjs` tente de remettre la précédente version Worker. Les migrations D1 déjà appliquées restent en place : ce rollback de code ne restaure pas la base. Concevoir des migrations compatibles et privilégier un correctif compensatoire ; une restauration D1 est une opération distincte à préparer et vérifier, pas une commande automatique de ce guide.

Pour un rollback ou retrait volontaire, faire relire et commiter un plan sous `infra/releases/<slug>.json`, puis déclencher `tools.yml` sur `main` avec `operation=rollback` ou `revoke` et le champ `plan`. Le plan fixe cible, état attendu et UUID de requête ; un retrait fixe aussi le hash et un document de révocation déjà signé avec compteur supérieur. Garder tous les tombstones. La syntaxe et les garde-fous réels sont dans [les plans de maintenance](../infra/releases/README.md).

Les entrées CLI sont `node scripts/rollback-release.mjs --plan <chemin>` et `node scripts/revoke-artifact.mjs --plan <chemin>`, dans leur contexte protégé. `promote-release.mjs` exige également un **plan** : les options génériques `--version ... --env production` du plan initial ne correspondent pas à son interface implémentée. Aucun plan production n’est fourni ni exécuté ici.

Une révocation connue reste bloquante, même avec un ancien ETag, une archive locale ou un canal expiré. Le rollback Desktop annonce un canal autorisé ; il ne force pas l’installation ou un downgrade. Conserver journaux expurgés, SHA, version IDs et bookmark avant d’intervenir. Les procédures complètes de restauration et leur drill restent l’étape 21.2, non accomplie par cette documentation.

## Supervision et clôture

La télémétrie à métadonnées bornées est implémentée et testée ; sa configuration staging est préparée, sans persistence de logs ni traces. Collecteur global, destinataire/astreinte et activation continue restent à configurer. Les compteurs R2/IA ne sont pas une facture. Procédures et seuils : [OPERATIONS](OPERATIONS.md), [alertes](../infra/alerts.md), [TELEMETRY](reports/TELEMETRY.md).

Archiver la CI du SHA retenu, les hashes des artefacts, les résultats de smoke, les preuves de signature et les réserves. Ne geler une RC ou V1 qu’une fois leurs gates satisfaits ; les documents de cette étape n’accordent pas cette validation.
