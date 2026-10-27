# Étape 6.2 — Capacités et stockage natifs

Date : 2026-10-27. Base : 9c3e2d3. Résultat : 269d7a2 (implémentation du lot). Lot 6.2, 6.3 et 7.0 explicitement autorisé par « Enchaîne les 3 prochaine étape ».

Le guest signé peut désormais émettre des CapabilityRequest SDK via host_enqueue. Le moteur valide UUID, taille, capacité déclarée, doublons, file totale de 128 demandes et génération active. Aucun champ de permission ou d’identité fourni par le guest ne crée de grant. Le Core fournit contexte et déclarations ; Gateway contrôle chaque effet. Le shell affiche capacité, portée et durée, propose refus/accord, et délivre capability.result avant de rafraîchir le rendu. Un accord de session est réutilisé ; un accès nécessitant un geste exige toujours un nouveau clic hôte. Fermer révoque contexte, handles et travaux non engagés.

Backend réel : rfd pour sélection/sauvegarde OS ; fichiers ouverts et conservés par handles opaques liés au contexte, 10 Mio maximum et lectures de 256 Kio. Aucun chemin guest n’est utilisé pour ouvrir un fichier. Les jonctions et symlinks sont refusés, y compris les ancêtres ; les erreurs de vérification échouent fermées. Une sélection vide est annulée sans effet. La sauvegarde utilise fichier temporaire créé exclusivement, sync_all et rename.

Stockage : répertoire résolu par dirs::data_local_dir, puis kortex/tools/<identifiant validé>. Le profil est haché pour le nom du record ; les clés restent des données. Révision comparée sous verrou OS, attente bornée à trois secondes, remplacement atomique, checksum et format interne versionné. Quota logique 5 Mio, record physique borné à 24 Mio, versions et migration N-1 via ToolStorage. Ce sont des données non secrètes ; les jetons restent hors de ce backend (14.5). Les contrôles protègent contre les chemins du guest ; aucune résistance absolue à un processus local privilégié modifiant simultanément les répertoires n’est revendiquée.

Réseau : HTTPS uniquement, origines choisies par configuration hôte, validation de toutes les IP par Gateway, connexion épinglée aux IP validées, proxy désactivé, TLS vérifié, aucun redirect automatique. DNS borné à dix secondes pour l’appelant, HTTP à trente secondes et réponse à 256 Kio. Une redirection externe sans droit est rejetée avant connexion. Presse-papiers raccordé par arboard, sous permission et geste. Audit borné et expurgé en mémoire.

| Commande | Sortie / résultat observé, Windows |
|---|---|
| `cargo test -p kortex-runtime-native -p kortex-capabilities -p kortex-core --locked` | 0 ; 48 tests réussis, un test Internet exclu de la suite hors ligne |
| `cargo test -p kortex-runtime-native --locked --test capabilities live_https -- --ignored` | 0 ; HTTPS réel example.com et redirect externe httpbin.org refusé |
| `cargo clippy -p kortex-runtime-native --all-targets --no-deps -- -D warnings` | 0 ; tous les targets de la crate |
| `powershell -NoProfile -ExecutionPolicy Bypass -File tests/desktop/native-smoke.ps1` | 0 ; deux parcours réels Desktop |

Tests : persistance/réouverture, outils et versions isolés, conflits, migrations et rollback, quotas, corruption, traversal, jonction Windows réelle, annulation, expiration des handles, session/once/révocation et audit expurgé. La validité du geste est vérifiée au début ; sa consommation ne limite plus un sélecteur OS à une seconde. Un test spécifique prouve ce comportement, sans renouveler le geste.

Échecs corrigés : conversion d’erreur getrandom, chemins des fixtures communes après déplacement, déclaration locale Rust const/let et style Clippy. La place disque a nécessité un cache de compilation local sur D:, sans changer la configuration partagée ni supprimer de source. Les dialogues rfd sont raccordés et compilés ; leur interaction OS n’est pas automatisée. Le presse-papiers réel de l’utilisateur n’a pas été modifié par les tests. Un IO système déjà engagé peut finir jusqu’à sa limite ; aucune annulation instantanée du DNS système ou d’une fenêtre OS n’est promise.

Preuves : tests/capabilities.rs de la crate, tests/native/capabilities.cjs, docs/evidence/native-capabilities.png et validation-6.2-7.0.json. Aucun schéma gelé modifié.
