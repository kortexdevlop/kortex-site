# Gate phase 7 — Référence calculatrice v1

Date : 2026-10-27. Base : 17eb70a. Résultat : 84b0633. Vérification technique et rédaction : Codex. Autorisation de réaliser les trois étapes : demande directe du propriétaire « Enchaîne les 3 prochaines étape ». Cette trace ne constitue pas une signature humaine indépendante ni une attestation de sécurité.

**GO local : référence outil 1 figée pour poursuivre vers le packaging.** Périmètre démontré : Windows/Wasmtime/WebView2, Chromium et WebKit. Linux/macOS et Firefox restent exclus sur instruction du propriétaire, comme dans les gates précédents.

| Critère | Preuve |
|---|---|
| Déclarations respectant ABI/UI 1.0 | Trois tests de déclaration et 46 cas de contrats communs |
| Calculs dans le guest, clics et actions communs | State::calculate ; 16 cas numériques et parcours réel |
| Module pure compute | Zéro import constaté par Wasmtime ; permissions=[] |
| Même paquet signé, aucune voie unsigned | ToolLoader/TrustStore et verify_package normaux, empreinte commune |
| Réponses et rendus compatibles | 13 goldens exécutés SDK, Worker et natif |
| Erreurs, clavier numérique et reprise | UI/action 1200 et DIVIDE_BY_ZERO ; Entrée sur bouton ; cancel/close/reopen |
| Régression globale | 80 tests Rust réussis, 28 adaptateur Web, 10 shell/E2E ; un smoke Internet ignoré explicitement |
| Contrat figé conservé | Cinq empreintes de schémas inchangées ; aucun nouveau message ou prop UI |
| Réutilisation pour prochain outil | Guide TOOL_SDK, générateur de fixture paramétrable, aucune branche d’identité dans les hôtes |

Référence binaire et filaire : tests/golden/calculatrice-reference.json. Guest : f095744f63b9594af61c02a6a515f2ad7ddab0ca3d5e7ccbcc9120a78f850c50. ZIP : fac58c7af499a828516f7d5ab04d7d0541faa0e6af8ad3aad3f8727aebca35d4. Corpus : 5c73355801ae1d56c1ad589999b0bdce1682ab8f88122b962ec7c09472af3b22. La signature test et la clé publique sont consignées dans l’observation structurée ; la clé privée n’est jamais persistée. Les tests refusent une dérive et ne régénèrent pas leurs attentes. Un changement de compilateur/chemin peut modifier le binaire et exige réexamen de la référence ; ceci n’est pas une preuve de reproductibilité multi-OS.

Les interactions ont découvert un défaut générique de sérialisation Web : Map JavaScript au lieu d’objet JSON. Sa correction sert tous les outils et conserve le protocole. La CSS partagée garantit aussi 44 px de largeur minimale des boutons. Aucun changement du moteur pour reconnaître la calculatrice, aucune logique métier hôte. Ces corrections transversales sont assumées dans le diff et couvertes par les régressions.

Limite du design : les raccourcis globaux Entrée et Échap/focus envisagés en 7.0 ne sont pas disponibles dans les événements figés. Le lot conserve le contrat : saisie normale, tabulation, Entrée/Espace sur boutons et remise à zéro explicite. Le test vérifie Entrée, pas un raccourci global. La livraison ne revendique pas ces fonctions absentes.

La fixture signée est un moyen de test local, pas une publication. Pas de cloud déployé, de clé de production, de packager phase 8 ni de compatibilité des plateformes exclues. Le contrôle réseau Windows distingue les événements virtuels internes Dioxus et les requêtes externes ; il ne surveille pas l’ensemble de Windows. Le smoke HTTPS ignoré était déjà optionnel et a été exercé dans le lot précédent, sans constituer une preuve nouvelle de ce gate.

Décision enregistrée : gel TOOL1_V1 local, sous le périmètre ci-dessus. Rapports STEP-7.1/7.2/7.3, preuves evidence/tool-1-parity.md et evidence/validation-7.1-7.3.json. Fin du lot autorisé ; 8.0 reste à démarrer.
