# Ordre de déploiement

1. Point de restauration Time Travel D1 ; migrations compatibles uniquement en avant.
2. API Worker : préserver les bindings, paramètres, quotas IA et secrets existants ; vérifier health, catalogue et téléchargement.
3. Site Astro et Runtime Web isolé : le Service Binding API doit répondre ; publier les assets hashés et le bootstrap SRI ; contrôler le Runtime déployé.
4. Publier le document signé des canaux après comparaison du hash de bundle compilé et déployé. stable reste nul tant que la production n'est pas approuvée.
5. Desktop : installer Windows staging testé, préversion immuable, provenance signée, puis mise à jour du canal Desktop signé.

Les workflows exigent main, le dépôt/identifiant attendus, la validation du même commit et l'environnement protégé. L'authentification de l'admin API est un OIDC GitHub court, pas un token opérateur durable. La clé racine est hors CI ; la clé de release staging est déléguée par un certificat public signé.

En cas d'échec du smoke, `ci-deploy.mjs` revient à la version précédente du Worker. Il n'exécute jamais une migration SQL inverse ou une restauration destructrice D1. Les anciennes versions doivent donc conserver la compatibilité avec le schéma courant.

Les exercices réels `rehearse-staging-rollback.mjs api|runtime --operator` passent par la version précédente, contrôlent le service et restaurent immédiatement la version courante avec ses bindings. L'identité de schéma D1 est comparée avant/après. Les preuves pipeline-rollback-* ne concernent que le staging. Aucun DNS, routage mail ou service production n'est touché.

Les versions de Runtime ont une séquence signée croissante ; les clients refusent un document ancien. Le publisher refuse une version sémantique inférieure. Un rollback d'urgence doit être une action opérateur explicite et garder les outils compatibles, jamais une réinstallation automatique plus ancienne chez l'utilisateur.
