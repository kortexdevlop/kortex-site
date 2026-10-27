# Étape 8.5 — Gate et gel PACKAGE_FORMAT_V1

Date : 2026-10-27. Base du lot : 02b0f6d. Résultat : 8ffb37d. Prérequis : 8.4 satisfait. Autorisation du propriétaire : « Enchaîne les 4 prochaines étapes », couvrant le gate et sa poursuite locale ; aucune signature de revue indépendante inventée.

Le paquet construit par le pipeline est admis par les deux hôtes normaux avec hash SHA-256, preuve B.2, racine hôte et snapshot signé. Les règles ZIP, métadonnées et ABI statique sont alignées ; aucune exception calculatrice. L'application Windows reçoit la confiance en configuration hôte. Les tests du moteur bas niveau peuvent fabriquer des guests invalides via Core ; l'application et ce gate passent par la validation complète.

| Commande | Date / environnement | Sortie | Résultat |
|---|---|---:|---|
| cargo test --workspace --locked | 2026-10-27, Windows | 0 | 98 tests réussis, 1 smoke HTTPS externe opt-in ignoré |
| pnpm --filter @kortex/web-adapter test | Windows, Chromium/WebKit | 0 | 32 tests réussis, dont les nouveaux refus ; corpus généré par la préparation native commune |
| pnpm exec playwright test -c tests/web/playwright.config.ts | Windows, Chromium/WebKit | 0 | 44 tests, dont paquet pipeline, signatures, refus hostiles et anciennes références |
| powershell -NoProfile -File tests/e2e/package-desktop.ps1 | Windows, WebView2/Wasmtime réels | 0 | UI/actions=1200, DIVIDE_BY_ZERO, annulation/fermeture/réouverture |
| node --test tests/publication/golden.test.mjs | Windows | 0 | Six schémas et octets de référence inchangés |

PHASE8_GATE.md porte la revue détaillée. CONTRACTS.md gèle PACKAGE_FORMAT_V1, le schéma artifact et l'immutabilité ; DECISIONS.md enregistre ADR-007. Les cinq anciens schémas et goldens TOOL1_V1 restent inchangés. tests/golden/calculatrice-artifact.json fixe le ZIP canonique, son module et le schéma ; il n'est pas régénéré par les tests.

Incidents du banc corrigés : démarrage CDP Windows trop tôt (attente bornée) puis rapport JSON lu avec encodage ANSI par PowerShell 5 (lecture UTF-8 explicite). Le parcours final passe. Le contrôle Clippy strict élargi a rencontré un avertissement préexistant kortex-ui ; il n'est pas déclaré vert. Clippy strict des trois crates de distribution passe ; contrôle des hôtes sans dépendances passe avec huit avertissements Desktop préexistants, sans changement de ces fonctions dans ce lot. Aucun déploiement ni audit tiers. Linux/macOS et Firefox exclus. Critère local satisfait ; limites conservées dans le gate.
