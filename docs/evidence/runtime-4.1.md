# Cœur Runtime 4.1 — preuve locale

23 octobre 2026. Windows, Rust/Cargo 1.99.1. Base bb0a4f7. SDK 4.0 approuvé par « Oui continue ».

## API et comportement observé

Runtime<E,C> coordonne un GuestEngine asynchrone et une Clock monotone injectés. Aucun Wasmtime, Dioxus, Desktop, tokio ou service cloud dans cette crate. Le cœur compile aussi pour wasm32-unknown-unknown.

load reçoit un VerifiedTool non constructible par un struct literal public. verify valide manifeste/contrat et identité, taille d'archive, SHA-256 et Ed25519 strict avec une table de clés de confiance injectée par l'hôte. Aucun constructeur unsigned de production. Le descripteur utilise le texte exact B.2 ; SHA2 0.10.9 et ed25519-dalek 2.2.0 exacts verrouillés (version disponible et rust-version 1.81 observée via cargo info ; build réel avec Rust 1.99.1 natif/WASM).

VerifiedTool atteste l'archive signée et l'identité : il ne prouve pas encore que les métadonnées séparées correspondent aux fichiers internes. L'adaptateur GuestEngine::load MUST vérifier/extracter les octets signés, comparer manifeste et contrat internes aux candidats fournis et vérifier le module/ABI avant de rendre un handle. Les clés de confiance proviennent de la configuration hôte ; elles ne doivent pas être reprises d'un paquet réseau. Le parseur ZIP et la rotation/révocation de confiance seront livrés aux étapes prévues. Aucun parseur produit anticipé ici.

Lifecycle : Created -> Loaded -> Initialized -> Running, puis Stopped ou Error. init précède render ; un deuxième initialize ou une action avant Running est rejeté. Les échanges ont un request_id monotone unique dans le Runtime, sans valeur d'autorisation. File FIFO de 128 éléments, refus QueueFull et QueueBusy sans perte de réponse ; process_next rend chaque réponse au caller. Les schémas d'entrée/sortie d'action sont réellement validés. Le schéma d'événement est vérifié ; contrôle des clés de la UI vivante et médiation de capacités encore à raccorder aux étapes responsables.

Les erreurs structurées guest sont propagées comme Reply sans détruire une instance saine. Crash, corrélation incorrecte, sortie hors schéma et timeout déclenchent Error et unload. Le registre retire toujours l'instance lors d'un shutdown ; les erreurs de déchargement restent visibles, combinées avec l'erreur primaire via CleanupFailure. Cela ne prétend pas récupérer une ressource qu'un adaptateur fautif n'aurait pas libérée.

## Délais et annulation

Le budget par action vient de contract.timeout_ms, 1..30000. Le délai commence au dispatch, pas au dépôt dans la file. bounded confronte la future d'exécution à une future de minuterie, vérifie l'heure avant/après réponse, abandonne la future expirée puis décharge le handle. L'adaptateur MUST être non bloquant, arrêter le travail au drop de future et posséder des handles RAII nettoyables sur annulation ; une future native qui bloque son poll ne peut pas être préemptée par ce cœur abstrait. Les Workers et interruptions Wasmtime seront raccordés dans leurs hôtes. Aucun délai réel d'un moteur produit n'est revendiqué ici.

## Tests exécutés

cargo test -p kortex-core --locked : dix tests réussis. cargo check -p kortex-core --target wasm32-unknown-unknown --locked : code 0.

- transitions légales/refusées, handshake puis actions/events/shutdown ;
- 128 demandes FIFO, 129e rejetée, aucune réponse perdue ;
- délai de 30000 ms avec horloge simulée, future d'exécution réellement abandonnée et unload appelé ;
- crash/corrélation/sortie incorrecte : registre vidé et unload appelé ;
- erreur guest normale conservant l'instance ; shutdown en erreur nettoyant le registre ;
- deux instances isolées, entrée/action/event invalides non envoyés au moteur ;
- signature altérée, contenu altéré et clé inconnue rejetés par vraie vérification cryptographique ;
- échec de nettoyage visible, registre vidé. Ce dernier test a d'abord échoué puis conduit à corriger la propagation d'erreur avant le passage final.

Le moteur et l'horloge de tests sont explicitement des stubs unitaires ; aucun service réel simulé en production. L'archive signée de test est un texte fixe, sans prétention de ZIP valide ; la vérification cryptographique est réelle. Les tests ne remplacent pas les preuves 3.0/3.1 des moteurs concrets.

Pas de changement de signature ABI, schéma ou contrat gelé ; aucune permission implémentée par anticipation. Aucun test Linux/macOS/Firefox supplémentaire.
