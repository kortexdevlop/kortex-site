# Étape 16.0 — Moteur de parcours local

Base : `2a7f367`. Date : 28/10/2026. Autorisation : propriétaire, « Enchaîne ces 3 étapes ».

Crate `kortex-workflows` : décodage du schéma gelé V1, profil 50 étapes/100 liens/50 niveaux/1 Mio, tri stable et rejet des cycles, doublons et dépendances manquantes. Registre construit uniquement depuis des `VerifiedTool`, identités/versions/instances/empreintes et contrats épinglés.

Mapping JSON Pointer : champs/indices/échappements vérifiés, recouvrements refusés, comparaison conservative des types et bornes, puis validation effective des entrées/sorties avant chaque invocation. Résultats en mémoire bornés (1 Mio par sortie, 4 Mio cumulés), journal sans valeurs privées. Exécution séquentielle avec délais, annulation d’une future pendante, arrêt sur erreur ; retry explicite limité à deux et réservé aux actions idempotentes.

Le port hôte impose le dispatcher normal et la politique de capacités. Aucune seconde voie d’exécution ni moteur cloud. Le raccordement produit est réservé à 16.3 ; le test d’intégration appelle déjà réellement le Runtime/Wasmtime et son paquet signé.

Validation : 13 tests du moteur dont graphe/mapping/retry, consentement et intégration signée ; 16 capacités et 21 cœur en régression (50 Rust distincts). Recontrôles ciblés après bornage du consentement et journal d’attente réussis. Paquet calculatrice réel : 1200 puis 1210, sans cloud. 31 tests Web, 3 contrats, analyse/build et essais éditeur référencés dans l’agrégat.

Chemins : nouvelle crate et tests, membres/lock Cargo, contrats/documentation. Schémas gelés inchangés. Décision : **terminée et close techniquement**. Preuves : `workflows-validation.json`, `workflows-runtime.json` et logs. Revue humaine non inventée ; Google 14.5 reste en attente.
