# Étape 2.0 — Contrats publics candidats

23 octobre 2026. Windows, Node 24.15.0. Base 33dc4c5.
Prérequis : 1.4 approuvée par « Oui, continue », activation distante des protections laissée en attente avec accord explicite.

Créations : docs/CONTRACTS.md ; cinq ADR 0001 à 0005 ; ce rapport.
Modifications : docs/DECISIONS.md ; docs/PROGRESS.md.
Les neuf blocs B.2 sont repris intégralement et identifiés KTX avec owners de rôle. Les dix ADR sont transcrits, avec correspondances entre fichiers et décisions. Les règles de compatibilité, dépréciation, erreurs et exceptions sont explicites. Aucun contrat gelé n'est modifié ; norme candidate seulement.

Vérification humaine obligatoire des noms/types/limites : non exécutée par l'agent, attendue lors de la revue du propriétaire. Aucun résultat de test runtime ajouté à cette étape documentaire.
Limites : responsables humains nominatifs à désigner ; protections GitHub toujours non activées ; essais Linux/macOS omis sur instruction, Firefox non vérifié. Aucun gel déclaré.
Statut awaiting_approval. Commit résultat : commit contenant ce rapport.

Résultats : comparaison automatisée des neuf blocs B.2 au plan : conforme, dix ADR couverts, aucun TODO (code 0). node scripts/check-progress.mjs : code 0, six étapes ordonnées. node scripts/check-env.mjs --example : code 0. node --test tests/config/secrets.test.mjs : six réussites, code 0. git diff --cached --check : code 0. La comparaison textuelle ne remplace pas la revue humaine prévue.
