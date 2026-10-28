# Rapport 14.2 — Modèle de synchronisation

28/10/2026. Base 60a7986. Autorisation : « Enchaine maintenant 14.2 jusqu'à 14.4 ».

Migration additive 0008 et historique conservé. OR-set de favoris avec tags immuables, retraits observés, tombstones, révisions serveur et CAS transactionnel. Consentement absent par défaut ; préférences strictement bornées. Aucun fichier utilisateur. Événements minimaux à rétention bornée.

Tests D1 : deux appareils depuis une révision offline commune, rebase sans perte, ajout concurrent conservé, ancien ajout jamais ressuscité, course CAS, conflits de préférences et quotas. 71 tests API du lot, 18 tests Web, types/builds et parcours réel staging réussis. Guide docs/SYNC.md ; preuves/hashs docs/evidence/account-14.2-14.4-validation.json. Commit et clôture technique dans PROGRESS.md, approbation humaine non inventée.
