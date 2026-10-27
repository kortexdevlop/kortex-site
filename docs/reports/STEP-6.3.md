# Étape 6.3 — Stabilité et portabilité native

Date : 2026-10-27. Base : 9c3e2d3. Résultat : 269d7a2 (implémentation du lot). Autorisation : lot de trois étapes, avec exclusions Linux/macOS et Firefox déjà explicites.

Windows est vérifié avec l’exécutable réel. Les scénarios couvrent ouverture/fermeture, 1200 pour multiply(25,48), erreurs de JSON/archive/ABI visibles, consentement guest et reprise après échec sans redémarrer l’hôte. Le contexte défaillant est libéré. Les demandes d’une ancienne génération ne passent pas dans l’instance suivante. Les réponses Wasmtime utilisent des oneshot et le timer Desktop une attente asynchrone, sans boucle de réveil actif. Drop de dispatch interrompt le moteur ; les tâches IO observent leur signal d’annulation.

| Commande | Résultat Windows |
|---|---|
| `cargo test -p kortex-runtime-native -p kortex-capabilities -p kortex-core --locked` | 48 réussis ; test Internet exécuté séparément |
| `cargo build -p kortex-runtime-desktop --locked` | Sortie 0 |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tests/desktop/native-smoke.ps1` | Sortie 0, deux parcours UI réussis |
| `pnpm exec playwright test -c tests/web/playwright.config.ts web-adapter/tests/capabilities.spec.ts -g 'guest SDK réel'` | 2 réussis, Chromium et WebKit |

Le scénario SDK Rust utilise le même fichier guest d’entrée et compare le rendu JSON attendu resolved:123 ; le scénario Web utilise ses métadonnées d’admission signées. Le guest Core conserve exactement son checksum et ses résultats, sans recompilation. Le journal est expurgé et borné ; seule une erreur constante de récupération est écrite en console.

Matrice et dépendances : docs/evidence/native-platforms.md. Tests exécutables/checklist : tests/desktop/native-smoke.md et native-smoke.ps1. Windows 11 Entreprise build 22621, Rust 1.99.1, Dioxus 0.7.10, Wasmtime 50.0.0. Installer et CI restent en phase 19.

Statut : validation technique locale Windows, dérogation de progression pour la matrice globale. Linux/macOS ne sont pas testés et le gate trois OS de phase 20 reste non satisfait. Le registre conserve awaiting_approval avec waiver, sans prétendre valider la matrice. La dérogation déjà autorisée permet de démarrer 7.0 ; aucun nouvel accord n’est demandé.
