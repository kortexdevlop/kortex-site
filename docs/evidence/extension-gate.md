# Gate extension 12.6 — 28/10/2026

Réussi : un sixième outil de test extérieur à tools/ est construit par le CLI canonique avec --source-dir, validé, signé et vérifié, puis publié réellement dans le catalogue de staging. La page générique et l’action echo fonctionnent sans nouvelle branche d’identifiant. Le test navigateur charge le vrai paquet signé.

Les empreintes website/src, runtime et api/src sont identiques avant/après la publication. Les identifiants de déploiement du site et du Runtime sont identiques pendant la publication et le test. Une clé publique de test a été provisionnée avant cette mesure, puis retirée après nettoyage ; cela ne constitue pas une absence de configuration de confiance. Les sources du cœur et des composants existants sont inchangées par ce gate (contrôle git diff sur la base 6a47c5e).

Nettoyage réussi : outil révoqué via l’API d’administration, absent du catalogue (404), téléchargement refusé (410), objet temporaire R2 supprimé. Les traces d’audit append-only de la publication/révocation sont conservées. Le modèle de fixture demeure uniquement dans tests/integration, jamais dans les outils produit. Politique opérateur et confiance publique Runtime restaurées. DNS, production et messagerie intacts.

Preuves : extension-staging.json, extension-playwright.json, extension-staging.png, extension-artifact.json. Commandes : node scripts/prepare-extension.mjs ; node scripts/test-extension-staging.mjs ; cargo test -p kortex-xtask --locked --offline. Test réalisé sur Chrome système, Windows ; exclusions Linux/macOS/Firefox conservées. Cette preuve démontre l’ajout sans modification produit, pas une charge de 10 000 outils.

Nettoyage de la clé de test : révocation auditée dans D1 également confirmée (fixture_signing_key_revoked), en plus de sa suppression de la confiance active du Runtime. Les audits immuables empêchent de republier cette même version de fixture : un nouvel identifiant ou une nouvelle version est nécessaire pour une nouvelle exécution complète.
