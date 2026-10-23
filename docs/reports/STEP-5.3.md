# Étape 5.3 — Chargeur .tool signé Web

Base 5ae8d6a ; démarrage autorisé par « Ok, continue », achèvement demandé par « Termine l’étape ».

TrustStore : clés publiques hôtes copiées, dates de validité, révocation, SHA-256 sur octets ZIP exacts et Ed25519 strict B.2. Aucun mode non signé ; clé inconnue/expirée/révoquée refusée. VerifiedTool.verifyArchive ne construit le jeton privé qu'après signature, extraction bornée et validation des schémas/identités/version/ABI. Les sections WASM personnalisées ne sont pas nécessaires pour le ZIP : les métadonnées internes sont déjà couvertes par la signature de l'archive complète.

Archive : ZIP classique mono-disque, méthodes stored/deflate, pas de chiffrement, ZIP64 ou data descriptors ; noms relatifs ASCII bornés, pas de traversal/backslash/segment vide/doublon insensible à la casse ni symlink ; cohérence central/local, offsets et absence de chevauchement, CRC32 ; <=256 entrées, archive <=25 Mio, entrée <=20 Mio, total décompressé <=50 Mio, ratio <=100. Décompression en flux interrompue dès dépassement de la taille déclarée. manifest.json, contract.json et module.wasm obligatoires. Aucun fichier extrait sur disque ni HTML outil exécuté.

Téléchargement : origine API fixe, HTTPS (exception boucle locale de développement), aucun credential/redirect, MIME zip/octet-stream, taille contrôlée sur Content-Length et flux, délai 30 s et annulation. Le cache d'origine conserve au plus une archive de 25 Mio via un emplacement unique ; tout hit est revérifié cryptographiquement. Cache corrompu supprimé puis refusé, sans exécution ni repli non signé. Changer de version/hash remplace ce cache. Désinstallation nettoie le cache et ferme l'outil via l'hôte.

Raccordement : BrowserHost.loadTool affiche downloading/verifying puis démarre exclusivement VerifiedTool ; fermeture/chargement concurrent annulent le téléchargement précédent. Les états Dioxus sont visibles. L'API canonique reste /adapter/host.js. runtime/web/public/trust-keys.json contient une liste test vide volontaire : aucune clé de distribution réelle installée ; la confiance doit être fournie explicitement par l'hôte. Aucun secret client.

Validation : TypeScript strict et build dx réussis ; quatre nouveaux tests Chromium/WebKit couvrent signature exacte, altération, clé inconnue, expiration/révocation/ABI ; vrai ZIP signé extrait et exécuté (multiply=21), traversal, archive surdimensionnée, cache corrompu et mauvais MIME refusés. Régression complète : 26 tests adaptateur et 6 tests Web. Les tests d'archives de cette étape utilisent stored ; la branche deflate utilise DecompressionStream natif avec bornes et CRC, sans prétendre disposer ici d'un corpus exhaustif de compresseurs ZIP. Les formats non pris en charge sont refusés.

Aucun contrat gelé modifié. Linux/macOS et Firefox omis selon instruction. Preuves : docs/evidence/web-5.3.md et web-adapter/tests/loader.spec.ts. Étape awaiting_approval ; aucune étape 5.4 démarrée.
