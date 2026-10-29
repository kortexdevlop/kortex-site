# Rotation et compromission des clés

Séparer racine hors ligne, clé de signature déléguée, secrets Cloudflare, identité OIDC GitHub, fournisseur Google et sessions utilisateurs. Changer une seule de ces identités ne révoque pas automatiquement les autres.

## Clé déléguée

Suspendre les publications, retirer son accès CI, déterminer la dernière version/empreinte saine et produire avec la racine fiable un nouveau snapshot de clés et un document de révocation à compteurs supérieurs. Conserver les retraits connus. Provisionner une nouvelle clé déléguée dans l'environnement protégé et distribuer ses documents publics, puis tester une archive valide, une archive altérée, la clé retirée, le rejeu et l'expiration. Les graines privées ne vont ni dans Git, ni dans les logs, ni dans un environnement applicatif.

## Racine hors ligne

Une racine compromise ne peut pas certifier fiablement sa propre remplaçante. Arrêter la chaîne, ouvrir une demande de changement et désigner le canal de confiance indépendant pour distribuer une nouvelle ancre et, si nécessaire, une nouvelle version des hôtes. Ne pas accepter une racine fournie par le guest ou baisser un minimum de snapshot pour débloquer un client. Faire valider le plan d'invalidation, de migration et de rollback avant publication.

## OIDC et jetons

En cas de compromission d'un workflow, fermer l'accès à l'environnement et révoquer les jetons/clefs affectés chez leur fournisseur ; contrôler les claims exacts et le dépôt numérique. Pour un client Google compromis, désactiver la connexion correspondante et renouveler son secret côté serveur ; le Desktop PKCE ne reçoit pas ce secret. Révoquer les sessions/appareils exposés, conserver uniquement des traces expurgées et tester le parcours réel après rotation.

Une restauration D1 peut réintroduire un ancien état de clé/session : réconcilier avec les preuves de sécurité fiables avant de rouvrir. Les programmes de signature et formats publics sont décrits dans [SIGNING](../SIGNING.md) et [CONTRACTS](../CONTRACTS.md) ; ce document ne génère ni n'envoie aucune clé.
