# Étape 7.2 — Même paquet sur Web et Windows

Date : 2026-10-27. Base : 17eb70a. Résultat : commit contenant ce registre. Deuxième étape du lot autorisé.

Un ZIP local signé par clé Ed25519 de test contient les vrais manifest/contract/module/icon de la calculatrice. Le générateur commun accepte les chemins de métadonnées ; seul le harness choisit l’outil. La clé privée reste en mémoire. Le même fichier et la même signature sont admis par les vérificateurs normaux Web et natif ; aucun mode unsigned, import sensible ni permission.

| Commande | Résultat |
|---|---|
| pnpm exec playwright test tests/e2e/calculatrice-web.spec.ts | 2 parcours réussis, Chromium et WebKit |
| tests/e2e/calculatrice-desktop.ps1 | Parcours WebView2 Windows réel réussi |
| cargo test -p kortex-runtime-native --test guest real_calculatrice --locked | Réussi ; ZIP signé réel, zéro import, 13 goldens et refus d’altération |
| Parcours Web et Desktop | Pavé et action : 25×48=1200 ; UI et action : DIVIDE_BY_ZERO ; fermeture/réouverture vide |

Les tests vérifient aussi activation du bouton Multiplier par Entrée et surface d’une touche ≥44×44 px. L’annulation du dialogue initial Windows laisse l’outil fermé. Pas de requête externe pendant les interactions ; le transport interne virtuel Dioxus/WebView2 //__events est compté séparément, sans prétendre mesurer le trafic global OS. Les hashes exacts sont consignés dans docs/evidence/tool-1-parity.md ; captures Chromium, WebKit et Windows inspectées.

Les deux moteurs exécutent les mêmes octets. Linux/macOS et Firefox restent exclus par le propriétaire, sans compatibilité revendiquée. Le script de test ne constitue pas le packager de publication de phase 8. Autorisation de poursuivre 7.3 dans le même lot.
