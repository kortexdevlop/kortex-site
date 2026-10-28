# Étape 15.2 — Médiateur local IA

Base : `50159f2`. Date : 28/10/2026. Autorisation : propriétaire, « Enchaîne ces 3 étapes ».

Revue locale stricte du schéma V1, des actions et des paramètres contre le contrat signé chargé. Aucune invocation à réception ; décision par étape, consommation unique, expiration cinq minutes, pin de l’instance/artefact, refus si outil fermé/modifié. Le cœur appelle la même méthode `Runtime::invoke_action` ; la passerelle Web utilise le même dispatcher `action.invoke`. Sorties limitées à 32 Kio, handles/chemins/binaires masqués, aucun envoi automatique des résultats au cloud.

Web : messages iframe liés à origine/fenêtre/nonce/session/corrélation ; aucune capacité accordée par une décision IA. Windows : paquet installé signé, contrôle de version publiée en ligne, aucune installation automatique. Les actions à permission/effet sensible sont refusées par l’assistant natif et renvoyées vers l’interface de l’outil, afin de conserver le consentement existant. Les cinq actions pures du lot fonctionnent hors connexion dans Wasmtime.

Validation : 21 tests cœur, test adaptateur Chromium sur vrai WASM signé (1200, refus/rejeu/expiration), deux tests natifs de pont et une matrice supplémentaire de cinq outils réels. Les 24 tests Desktop de la suite complète puis la matrice ajoutée donnent **25 tests natifs distincts réussis**. Les échecs historiques TCP Windows observés durant les premiers essais sont corrigés par un seul nouvel essai de lecture publique GET/HEAD sur les erreurs transport Windows 10053/10054 avant en-têtes ; aucun retry d’erreur HTTP, de corps, d’intégrité, de mutation ou de capacité. Les tests négatifs d’installation/révocation restent réussis.

Chemins : `crates/kortex-core/src/ai_bridge.rs`, export de module, tests ; `web-adapter/src/{ai-bridge,host,iframe,interop}.ts` et build ; `desktop/src/{ai_bridge,api}.rs` et tests. Schéma public inchangé, adaptation des flux existants. Aucun moteur cloud ajouté.

Décision : **terminée et close techniquement**. Preuves : `assistant-validation.json`, `assistant-native-matrix.json`, logs cœur/adaptateur/Desktop. Google réel (14.5) reste en attente ; l’autorisation explicite du propriétaire couvre ce lot. Aucun statut d’approbation humaine inventé.
