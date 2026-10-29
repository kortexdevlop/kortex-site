# Distribution, révocations et CI

29/10/2026 — Windows et Chromium uniquement.

Le dépôt public autorisé est https://github.com/kortexdevlop/kortex-site, identifiant 1437766566, branche main. Le compte du connecteur GitHub est différent ; aucune permission de ce connecteur n'est supposée pour ce dépôt.

## Révocations

Le contrat partagé Rust/TypeScript couvre la version monotone, la signature Ed25519 de la racine, les clés compromises et les triplets outil/version/hash. Une nouvelle liste conserve les anciennes révocations. La validité ne dépasse jamais 24 heures. Une révocation connue reste bloquante après expiration ; tout autre lancement exige une reconnexion après expiration. Le recul d'horloge est refusé. Le cache reste sous l'autorité du profil hôte ; aucune résistance à un administrateur local malveillant n'est revendiquée.

L'API refuse de servir une liste périmée, mal signée ou ne couvrant pas les révocations D1. Le Web contrôle la liste et le téléchargement avant admission, puis contrôle à nouveau avant de rendre l'outil au lanceur. Desktop contrôle installation, mise à jour et lancement. Les instances déjà en cours ne sont pas arrêtées rétroactivement.

La clé privée de révocation est chiffrée avec DPAPI pour l'utilisateur Windows, hors du dépôt, dans `%USERPROFILE%/.kortex-offline/revocations.dpapi`. `infra/trust/revocation-root.json` contient uniquement sa clé publique. Ne pas réinitialiser la racine existante ni copier sa clé privée dans GitHub ou Cloudflare.

**Exploitation obligatoire :** avant chaque expiration, l'opérateur actualise les tombstones D1, augmente la version du document et exécute `node scripts/offline-revocations.mjs --input <fichier> --out <fichier>` sur son poste de confiance, puis publie le document public signé dans `REVOCATION_DOCUMENT`. La signature quotidienne n'est pas automatisée avec la clé racine. En l'absence de renouvellement, le refus de lancement est volontaire. La révocation initiale couvre les clés et archives déjà révoquées en staging.

## Vérifications

`node scripts/ci-verify.mjs --lane contracts|native|web|security` exécute les commandes réelles et garde les journaux bornés dans `target/ci-reports`. Les jeux de données sont générés localement avec des signatures de test ; aucun ancien dossier target n'est requis sur un runner neuf. Le build Dioxus utilise un bootstrap externe et l'interface hôte globale initialisée avant le module compilé. Les modules de test WASM ont explicitement un plafond de 64 Mio.

`node scripts/security-gate.mjs --collect` lance Cargo/npm audit, produit les SBOM CycloneDX 1.6, l'inventaire des licences et une analyse AST TypeScript ciblée. Clippy est exécuté séparément. L'analyse AST n'est pas un audit universel. Les avertissements RustSec et licences nécessitant une revue sont exposés ; aucun accord juridique ou humain n'est inventé. `--release` reste bloqué tant que les gates de production sont ouverts, notamment Google réel et rétention opérateur de 18.2.

## Publication et déploiement

Les workflows séparent validation, packaging des outils modifiés, publication, déploiement API et site/runtime. Les actions sont épinglées par commit ; les jobs de publication refusent PR, fork, mauvaise branche, dépôt ou environnement avant d'utiliser les identifiants. Les clés d'objets R2 sont adressées par hash. Les reprises gardent le même identifiant de publication. La provenance in-toto est signée par la clé de release ; elle ne constitue pas une attestation GitHub/SLSA certifiée.

L’environnement staging GitHub est limité à main. KORTEX_STAGING_PUBLISH_ENABLED et KORTEX_STAGING_DEPLOY_ENABLED valent true. Le token Cloudflare et la clé de release déléguée sont configurés dans les secrets de cet environnement. La racine privée reste hors CI. Les certificats publics release-staging-root.json et release-staging-keys.json correspondent à la clé enregistrée dans D1 et dans les hôtes. L’API admin vérifie les claims OIDC du workflow tools.yml, le sujet immuable effectivement configuré par GitHub et sa signature RSA, y compris son en-tête x5t documenté. Les appels serveur utilisent redirect:manual et refusent explicitement les réponses de redirection ; redirect:error n’est pas accepté par le moteur Cloudflare déployé.

Le passage en public a été demandé par le propriétaire. Les anciens refus de facturation du dépôt privé sont historiques. Les runners publics exécutent réellement les jobs. L’environnement production exige maintenant le reviewer propriétaire ; il n’a ni secret Cloudflare ni ressources de déploiement configurées. Les scripts continuent de refuser toute livraison production.

Le déploiement API conserve les paramètres vivants, demande un point de restauration D1, applique uniquement les migrations en avant et contrôle le service après déploiement. Le site/runtime exige une API compatible. Un échec de smoke restaure la version précédente du Worker ; **aucune restauration destructive de base n'est automatisée**. Le retour arrière et la restauration des Workers API/Runtime ont été exercés réellement en staging ; voir les preuves pipeline-rollback-*.json. Le déploiement staging local est distinct de cette preuve CI.

Les preuves historiques restent intactes. Les preuves de cette phase utilisent le préfixe pipeline-. DNS, messagerie, production, secrets applicatifs existants et limites IA ont été conservés. Les secrets nécessaires à la CI staging ont été configurés dans son environnement protégé.

## Vérifications observées

Les 91 tests API et 27 scénarios Chromium passent localement sur Windows le 29/10/2026. Les essais réels couvrent les révocations, les métadonnées de canaux signées, les refus de downgrade/replay, la compatibilité ABI, les permissions et le module calculatrice fraîchement reconstruit. Les références binaires et goldens historiques restent inchangés.

La PR négative no 14 a effectivement échoué sur le contrat altéré ; elle est fermée sans fusion et sa branche a été supprimée. Les détails figurent dans pipeline-negative-pr.json. L’installation, le démarrage et la désinstallation de l’EXE Windows ont réussi ; le contrôle visuel WebView2 a vérifié les canaux signés. Le staging sert le Runtime Web 1.0.1 et le calcul signé donne 1200.

Les résultats finaux des publications GitHub sont consignés séparément dans les rapports STEP-19.* et les preuves pipeline-*. La production, la signature Authenticode, Google OAuth réel et la rétention opérateur restent soumis aux gates déjà documentés. Le canal stable reste nul.
