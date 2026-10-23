# Permissions et capacités 4.2 — preuve locale

23 octobre 2026. Base 2696d7f. Rust 1.99.1, Windows, cible WASM vérifiée. Prérequis 4.1 approuvé par « Oui continue ».

## Politique et identité

Les sept capacités V1 sont un enum fermé ; une capacité inconnue est refusée. Policy lie les déclarations et grants au tuple tool_id/version/instance_id et à une ressource canonique. Aucun effet offert par défaut : décision Deny. Ask n'est possible qu'après activation explicite de la demande de consentement ; un refus utilisateur explicite empêche les relances de prompt sur ce scope. AllowOnce est consommé par l'opération, AllowSession reste soumis à expiration et révocation.

ConsentPrompt fournit au futur hôte UI l'identité, la capacité, la portée, la durée et l'échéance. Il ne constitue pas une boîte de consentement déjà affichée : l'UI de plateforme doit montrer cette information et le contexte du fichier sélectionné avant d'appeler grant. Les appels de déclaration, grant, record_gesture et création de handles sont des API de l'hôte de confiance, jamais des commandes JSON guest autorisant elles-mêmes un effet.

FilePick, FileSave et ClipboardWrite exigent un GestureToken hôte frais (1000 ms) à usage unique, distinct du grant. Les tokens ont des champs privés et un identifiant CSPRNG ; le bridge ne doit accepter aucun token fourni par le guest. Une autorisation de session ne remplace pas un geste quand celui-ci est exigé.

Le Runtime fournit lui-même l'identité et les permissions du manifeste chargé à execute_sensitive ; le guest ne choisit pas un autre tool_id/version/instance. prepare_capabilities prépare les déclarations pour l'UI, execute_capability fait passer l'appel par Gateway, dispatch_capability_result valide la livraison asynchrone avec l'enveloppe gelée. shutdown_with_capabilities révoque grants, gestes et handles. Une instance retirée par erreur/crash ne peut plus appeler ce bridge. L'hôte doit révoquer sa session externe lors de ces fermetures et créer un Gateway propre pour un nouveau Runtime ; ne pas réutiliser des grants d'un Runtime détruit.

## Handles, budgets, journal

Handles à identifiants aléatoires 256 bits, opaques, liés au propriétaire complet, au droit Read/Write et à une durée <=300000 ms. Aucun chemin guest ne remplace un handle ; la référence backend reste locale à l'hôte, accessible seulement après validation. Les ressources storage sont des clés dans le namespace privé de l'outil ; l'adaptateur doit utiliser context.tool_id comme namespace, jamais une valeur contenue dans les octets du guest.

Budget fichier initial : 10 MiB par demande et résultat interne, configurable à la baisse ; les autres opérations restent bornées à 1 MiB. Un résultat dépassant max_output_bytes est rejeté. L'adaptateur doit limiter les lectures en amont avant allocation, pas attendre ce contrôle final. La limite de message JSON ABI de 1 MiB reste inchangée ; les gros fichiers devront transiter par handles/transferts bornés adaptés. Le test Runtime rejette un résultat de capacité JSON trop grand avant mise en file.

AuditLog local de 1024 événements maximum : date, instance, capacité, résultat catégorisé et nombre d'octets entrants. Aucun nom de ressource, chemin, URL, contenu, token ou texte d'erreur backend ; aucun envoi cloud implémenté. Tests avec noms et payloads simulés sensibles confirment leur absence du journal.

## Réseau et responsabilités de l'adaptateur

Une permission réseau vise une origine HTTP(S) canonique explicite, sans credentials ni fragment. Chaque redirection vers une nouvelle origine repasse par la politique ; au plus cinq redirections. Une autorisation once couvre la chaîne bornée de même origine d'une seule demande. Résolution DNS et IP littérales sont inspectées ; adresses privées/locales/réservées courantes refusées. L'échéance est revérifiée après résolution DNS et au retour du service.

AuthorizedCall est construit uniquement par Gateway ; il remet la ressource validée, les IP approuvées et l'échéance au driver. CapabilityHost MUST désactiver les redirections automatiques, épingler la connexion aux IP approuvées, respecter l'échéance et borner les données. resolve non implémenté retourne Unsupported ; aucune capacité n'est implicitement activée. Un futur adaptateur Web qui ne peut pas prouver ce comportement doit refuser l'opération ou utiliser un chemin contrôlé prévu par son architecture ; aucune garantie DNS spéciale du fetch navigateur n'est inventée. Aucun driver réseau, fichiers, clipboard ou stockage produit n'est livré à cette étape abstraite.

## Résultats réels

- cargo test -p kortex-capabilities --locked : 16 tests réussis.
- cargo test -p kortex-core --locked : 11 tests réussis, dont passage par policy avec identité manifeste, livraison asynchrone et révocation.
- cargo test --workspace --locked : 45 tests et un doctest réussis.
- cargo check -p kortex-capabilities -p kortex-core --target wasm32-unknown-unknown --locked : code 0.
- Contrats croisés : trois tests Node, 46 cas TS/Rust identiques ; schémas 41 contrôles ; conformité 26 cas.
- Cinq empreintes des schémas FROZEN_V1 comparées à CONTRACTS : inchangées.

La matrice couvre le refus non déclaré, défaut deny, ask sans effet, once/session/deny, expiration et révocation, tool/version/instance/resource différents, geste frais/expiré/rejoué, handles expirés ou appartenant à un autre outil, fichiers >10 MiB, réponse trop grande, sept capacités autorisées uniquement après policy, redirection non autorisée avant effet cible, seconde origine explicitement autorisée, boucle de redirection, IP locales/obfusquées/IPv6 et DNS privé, grant expirant pendant DNS, journal borné sans payload.

Les services et horloges des tests sont des stubs unitaires déclarés ; aucun effet système réel présenté comme service produit. Le premier ajout de test de redirections a eu une erreur de nom masqué dans le helper, corrigée avant passage final. Aucun schéma/ABI gelé modifié. Dépendances exactes URL 2.5.8 et getrandom 0.3.4 (wasm_js pour le côté hôte) ; les sous-paquets JSON Schema existants ont été maintenus aux versions de lock précédentes après résolution.
