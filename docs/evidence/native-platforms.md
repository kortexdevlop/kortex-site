# Matrice native — 6.3, 27 octobre 2026

| Plateforme | Compilation / exécution | Statut |
|---|---|---|
| Windows 11 Entreprise, 10.0.22621 | Rust 1.99.1 MSVC, Dioxus Desktop 0.7.10, Wasmtime 50.0.0 ; vrai exécutable et WebView2 | Réussi localement |
| Linux | Aucun essai effectué | Exclu par instruction du propriétaire, non validé |
| macOS | Aucun essai effectué | Exclu par instruction du propriétaire, non validé |

Version WebView2 observée sur le poste : 155.0.4283.62. Le rendu passe dans la WebView ; le WASM est exécuté dans Wasmtime natif. Le port CDP 9332 est activé exclusivement pour le banc de test puis son processus est fermé.

Scénario reproductible : tests/desktop/native-smoke.ps1 et native-smoke.md. Deux parcours sur l’exécutable : actions et fermeture ; guest SDK, refus/accord, callback, module incompatible et récupération sans relancer l’application. Captures native-desktop-6.3.png et native-capabilities.png. Le même scénario SDK Web passe sous Chromium et WebKit. Le test Rust parity.rs compare le nœud UI sérialisé attendu et utilise le même fichier guest SDK d’entrée ; les signatures et métadonnées d’admission restent propres à chaque paquet de test.

Le guest Core garde son checksum 3385c7f64a333da67e884b44f1260582d7ae5b5dd2921d033af7a1dce7548e65 ; 100 actions donnent 1200. La boucle CPU et l’annulation sont interrompues ; l’instance peut être détruite. La réception des réponses et le timer Desktop n’utilisent plus de réveil actif. Les files de capacités sont bornées et isolées par génération ; fermer ou recharger révoque l’ancien contexte et ses handles.

Le journal d’audit Gateway est local, en mémoire et borné à 1024 événements sans URL, chemins, clés ni valeurs. Les messages console de récupération se limitent à un code constant ; les erreurs détaillées de transport ne sont pas transmises au guest. Les captures de test ne contiennent que des fixtures non secrètes.

L’ancienne instruction « Ignore les essais Linux/MacOs et continue » et l’autorisation de ce lot permettent de poursuivre vers 7.0 sous dérogation, comme en 1.2. L’étape 6.3 n’est pas marquée validated pour la matrice complète. Le gate multi-OS de phase 20 reste non satisfait ; aucune compatibilité Linux/macOS n’est déduite de Windows. Aucun installateur ni runner CI n’est revendiqué.
