# Preuve 3.3 — iframe, Worker et ZIP hostiles

23 octobre 2026. Windows, Node 24.15.0, Playwright 1.64.1, Chromium 156 et WebKit 27.2. Firefox omis par décision du propriétaire, Linux/macOS également omis.
Commande : pnpm exec playwright test tests/spikes/sandbox-host --workers=1 --reporter=line --output=target/playwright-spike-3.3. Résultat final : trois tests réussis, code 0, durée 5.1 s.

Deux serveurs réels écoutent exclusivement sur 127.0.0.1 et des ports éphémères distincts. Le parent embarque le runtime avec sandbox allow-scripts allow-same-origin, sans allow-top-navigation. Les messages vérifient origine exacte, source de fenêtre et nonce aléatoire de session ; aucun targetOrigin générique. Les serveurs sont fermés en fin de test.

Essais réels Chromium/WebKit :
- lecture du document parent par iframe : SecurityError ;
- message parent légitime : acquitté ; faux nonce : rejeté ; iframe sœur de même origine et même nonce : rejetée comme source ; auto-message runtime : rejeté ;
- CSP nonce : script ajouté sans nonce non exécuté, requête externe bloquée avant réseau ; frame-ancestors restreint ; tentative top navigation : rejetée ;
- téléchargement réel de reference.tool, longueur et SHA-256 comparés aux octets du serveur ;
- Web Worker same-origin charge le binaire WASM 3.0, vérifie ses imports et l'instancie avec host_enqueue refusant les capacités ; typeof document vaut undefined et les exports ABI sont présents.

ZIP référence : 163083 octets stockés ; 56645 octets en version deflate. Manifeste et contrat canoniques et vrai module WASM 3.0. Le validateur de ce banc s'exécute sous Node, en mémoire, sans extraction disque. Il vérifie SHA-256 et signature Ed25519 détachée selon le texte B.2, avec une clé éphémère générée dans le test. Aucune clé produit ni publication.

Rejets exercés : signature absente ou fausse ; checksum modifié ; ../ et segments traversal ; lien symbolique Unix ; drapeau de chiffrement ; fichier caché ; ratio de décompression excessif ; payload ZIP 25 MiB+1 ; noms dupliqués ; divergence en-tête local/central ; contenu signé mais CRC invalide. ZIP référence stored et deflate acceptés. Le décompresseur impose maxOutputLength avant production des octets ; les tailles déclarées sont aussi contrôlées avant décompression.

Budget candidat : ZIP <=25 MiB ; total décompressé <=64 MiB ; entrée <=20 MiB ; ratio <=100:1 ; <=128 entrées. Ce banc refuse ZIP64, descriptions de données et multidisque. Ce n'est pas encore le parseur produit Web/native, ni un audit complet du standard ZIP. Pas de modification des plafonds B.2 ; politiques candidates explicitées dans CONTRACTS.

Mesure Node finale observée (pas un pic exhaustif, inclut Playwright et les fixtures malveillantes allouées par le test) : RSS avant 137895936, après 185905152 octets ; heapUsed avant 69173976, après 87670064 ; arrayBuffers après 29833909. Les budgets d'archives ne garantissent donc pas un processus hôte <=64 MiB. Aucun pic mémoire navigateur ou preuve sous pression mémoire faible n'est revendiqué.

P3 local : isolation et téléchargement prouvés dans deux moteurs ; vérification d'archives prouvée dans le banc Node. Le parseur final dans l'hôte navigateur, les seuils sous charge et les clés persistantes relèvent des étapes runtime/distribution. Aucun gel de portabilité globale.
