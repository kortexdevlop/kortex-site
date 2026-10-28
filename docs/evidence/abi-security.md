# ABI et ressources — 18.1

28/10/2026, Windows/Chromium. Limites renforcées sans modifier les messages ou exports ABI 1.0 : tables à maximum explicite ≤65 536, au plus une table ; mémoire linéaire ≤64 Mio, bornes de messages et pointeurs contrôlées avant copie. Pointeur d’entrée zéro légal rétabli côté Web, comme dans le contrat ABI et le moteur natif ; un contrôle signé valide le vérifie.

Native : 50 tests réussis, 1 smoke réseau public explicitement ignoré (aucun réseau public à scanner ici). Moteurs réels Wasmtime : huit guests hostiles alloc/dispatch/dealloc, overflow/overlap/trap ; quatre tests adversarial repris après contrôle supplémentaire des IP littérales. Paquets signés, cinq outils réels, stockage/fichiers/jonctions Windows. Core/ABI/UI : 37 tests réussis, dont queue 128, timeout, nettoyage, XSS échappé et refus de schémas UI invalides.

Web sécurité : 21 scénarios de base réussis et deux scénarios de confidentialité staging supplémentaires ; contrôle ciblé ABI repris avec pointeur zéro. Worker réel interrompu, ticker parent vivant ; neuf guests hostiles refusés et contrôle valide accepté. Corpus pointeurs : 4 096 mutations par moteur. Corpus ZIP/WASM statique existant : 10 000 mutations, zéro panic, zéro instanciation ; empreinte et référence historiques inchangées.

ZIP 9 conserve l’encodage canonique en fixant le système/permissions ; les goldens et archives historiques restent lisibles. Les fixtures V1 figées évitent qu’une publication 1.0.1 change artificiellement le corpus de référence ; le test natif de cinq outils utilise la nouvelle release signée valide pour la calculatrice plutôt que son ancienne clé expirée. Aucune expiration ignorée et aucun résultat métier simulé.

Preuves : security-native.log, security-native-recheck.log, security-core-ui.log, security-playwright.log, security-abi-web.json. Corpus et seeds : tests/security/abi-fuzz/README.md. Pas de cargo fuzz/libFuzzer exécuté ; ceci ne borne pas toute la RAM du processus/compilateur et ne prouve pas tous les modules possibles sûrs.
