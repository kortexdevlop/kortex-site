# Étape 18.1 — ABI, budgets et permissions

28/10/2026, base 6e67be3. Terminée et close techniquement pour le corpus Windows/Chromium ; statut humain formel en attente.

Audit et correction du budget des tables : maximum obligatoire ≤65 536, une table au plus, contrôle statique avant allocation guest. Mémoire ≤64 Mio, message ≤1 Mio, queue ≤128, action ≤30 s, timeout Worker/fuel/epoch et arrêt à la fermeture. Correction d’une divergence Web : pointeur zéro légal selon ABI, désormais accepté lorsqu’il désigne un message valide.

Vrais guests hostiles signés, aucun bypass d’admission : boucles dans alloc/dispatch/dealloc, piège, overflow, chevauchement, corrélation, tables/mémoire. Contrôle valide signé et 4 096 pointeurs mutés par moteur. Corpus ZIP/WASM fixe de 10 000 mutations inchangé et sans panic. Pas de campagne cargo fuzz prétendue. UI/XSS/queue/timeout/quota et fichiers réels couverts par les suites existantes reprises.

50 tests natifs et 37 core/ABI/UI réussis ; 1 smoke HTTPS externe explicitement ignoré, remplacé ici par les scénarios contrôlés sans scan public. 23 scénarios Web sécurité distincts passent, contrôle ABI ciblé repris ; 8 scénarios bureau passent sur staging durci. Voir abi-security.md et security-validation.json. Check Desktop Windows et build Web réussis. Aucune modification du format ABI ou des contrats guest ; aucune borne de toute la RAM du processus revendiquée.
