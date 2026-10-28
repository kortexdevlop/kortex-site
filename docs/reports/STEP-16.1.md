# Étape 16.1 — Consentement des parcours

Base : `2a7f367`. Date : 28/10/2026. Autorisation : propriétaire, « Enchaîne ces 3 étapes ».

Prévisualisation par étape, permissions agrégées, marquage des effets et confirmations obligatoires. Approbation d’étape ou de lot borné, liée au Run unique, à son document/contrats et aux bornes de retry ; expiration cinq minutes et consommation à usage unique. Le JSON du parcours ne peut ni fournir ni recréer ce consentement.

`AWAITING_CONSENT` conserve la position et les résultats en mémoire : une reprise ne rejoue pas les étapes terminées. Zéro invocation sans accord requis et sans grant réel du `Policy` partagé. Le consentement de parcours ne crée aucune capacité. Les scopes exacts et gestes restent contrôlés par le courtier normal.

Le Policy expose une révision de retrait, distincte de la consommation des grants à usage unique. Vérifications avant chaque nœud/tentative et après retour ; retrait pendant une opération ou durant une pause arrête la suite. Annulation/expiration/échec scellent le Run. Journal d’attente et d’effets possibles, aucune promesse de restaurer un fichier externe déjà modifié.

Validation : six tests spécifiques (grants réels absents, prévisualisation/union, batch/reprise/non-rejeu, changement de plan/action, retrait pendant effet/pause, expiration et annulation/timeout avec Drop), plus 16 tests capacités et 21 cœur. Suite moteur complète et vrai Run signé réussis. Aucun effet produit par les tests de refus ; reprise sans rejouer le premier nœud vérifiée.

Chemins : `permissions.rs`, runner, tests, révision additive interne du Policy. Aucun changement ABI/API ni de format public. Décision : **terminée et close techniquement**. Preuves dans `workflows-validation.json`. Le branchement hôte produit/UI est laissé à 16.3 conformément au périmètre autorisé.
