# Étape 6.1 — Shell Dioxus Desktop

Finalisation : 2026-10-27. Base : ceac36a ; résultat : 02f6736 (implémentation du lot). Troisième étape du lot demandé par le propriétaire. Prérequis : moteur natif 6.0.

Le shell Windows ouvre un .tool signé et son descripteur B.2 adjacent, avec des clés publiques hôtes fournies séparément. Il utilise Core, Wasmtime et le même UiBoundary que Web. Aucun App Store ni binaire natif d’outil. Le dialogue hôte propose d’annuler ou de continuer sans accès sensibles. Aucun consentement n’active de pilote absent : réseau, fichiers et presse-papiers du guest restent refusés jusqu’à 6.2.

Ouverture, action déclarée et fermeture passent par le Core. Les saisies invalides et erreurs d’admission sont visibles ; une initialisation échouée détruit l’instance. Le chargement limite le paquet à 25 Mio et refuse une version de descripteur inconnue.

| Commande | Environnement / date | Résultat |
|---|---|---|
| `cargo check -p kortex-runtime-desktop --locked` | Windows, 2026-10-23 | Sortie 0 |
| `cargo build -p kortex-runtime-desktop --locked` | Windows MSVC, 2026-10-23 | Exécutable généré, sortie 0 |
| `node tests/fixtures/make-tool.cjs` | Windows, 2026-10-23 | ZIP signé et clé publique éphémères |
| `node tests/native/desktop.cjs` | Exécutable Windows, WebView2, 2026-10-23 | Parcours réel réussi |

Le test pilote l’application exécutée : dialogue et annulation sans ouvrir le guest ; ouverture du ZIP signé ; même nœud data-node=result affichant Ready que Web ; clic sur multiply avec 25 et 48 donnant 1200 ; JSON invalide visible sans perdre le rendu ; fermeture supprimant le rendu ; archive altérée refusée avec shell toujours visible. Capture contrôlée visuellement : `docs/evidence/native-desktop.png`.

Reproduction : `runtime/desktop/README.md`. La WebView assure le rendu Dioxus ; le WASM s’exécute dans Wasmtime natif. Le test active temporairement le port diagnostic WebView2 9332 ; le produit ne l’active pas par défaut. L’application de test a été arrêtée.

Écarts : Linux/macOS et Firefox expressément exclus par le propriétaire. Des compilations ont échoué par manque d’espace ; après nettoyage des caches et symboles générés, compilation et tests ont réussi. Les erreurs RSX de développement ont été corrigées. Le shell reste expérimental ; pilotes et stockage durable natifs relèvent de 6.2, hors du lot.

Preuves : `tests/native/desktop.cjs`, `docs/evidence/native-desktop.png`, `docs/evidence/validation-5.4-6.1.json`. Aucun schéma ou ABI gelé modifié.
