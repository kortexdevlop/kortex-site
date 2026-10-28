# Étape 18.2 — Réseau, fichiers, tokens et confidentialité

28/10/2026, base 6e67be3. **Audit du périmètre actif réalisé ; étape partielle, non close intégralement.** Gates Google réel et politique de rétention opérateur encore ouverts.

Windows : adresses privées/mélangées refusées aussi par le backend, IP littérales privées/alias refusées, résolution fixée pour la connexion et recontrôle à chaque redirection. Test de DNS rebinding : réponse publique puis privée, une seule exécution avant refus. Web : DNS non vérifiable refusé UNSUPPORTED avant consentement/trafic ; IPv4 publique littérale seulement avec allowlist et permission, redirect error/credentials omit/réponse bornée. La CSP peut refuser plus strictement. Transport TLS local de test, aucun réseau public scanné.

Fichiers : jonction Windows réelle, handles opaques liés à l’instance, durée/quota, deux lectures simultanées et handle étranger, noms de sauvegarde hostiles sans effet. Auth : matrice Origin/CSRF/cookie dupliqué/propriétaire, sessions et tokens révocables, CAS et purge en cascade du propriétaire, compte B intact, erreurs/exports sans credentials. 88 tests API, dont deux nouveaux scénarios workerd/D1 ; authentification HTTP négative et CSP réels staging. Les 8 parcours du bureau passent encore.

Confidentialité : deux scénarios staging supplémentaires vérifient un vrai hash WASM privé, aucune donnée de fichier dans les requêtes, aucune inférence IA et aucune persistance parent ; proposition expirée sans exécution. PRIVACY.md inventorie durées logiques, stockage autorisé, effacement SQL et limites des sauvegardes/stockages hors ligne. data-flows.md relie les flux aux autorités. Les scans après mises à jour sont sans vulnérabilité détectée, avertissements documentés et non masqués.

Pour clore complètement : créer/configurer les clients Google Web/Desktop puis exécuter le vrai gate 14.5/18.2 (voir AUTH.md), et définir/vérifier la rétention de l’hébergeur avant production. Pas de secret à partager dans le chat. Aucune clôture universelle, revue humaine ou suppression physique instantanée des sauvegardes revendiquée. Production/DNS/mail/secrets/quotas préservés ; 18.3 non engagée.
