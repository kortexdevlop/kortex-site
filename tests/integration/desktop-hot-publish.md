# Publication à chaud Desktop

KORTEX_PHASE135=1 utilise la vraie API staging, D1, R2 et une application Windows déjà lancée. Le premier catalogue contient exactement les cinq outils publics et exclut l'identifiant aléatoire de la fixture. Le contrôle local authentifié ne publie sa version 1.0.0 qu'après les cinq cycles Windows. Il ne sert ni paquet ni réponse catalogue au client : les téléchargements du produit restent sur HTTPS Cloudflare.

L'application recherche la fixture, l'installe, exécute sa conversion Unicode, puis suit les versions 1.1.0 et 1.2.0. Le helper compare les octets du .exe avant lancement/après parcours. Aucun rebuild ou relancement pendant publication. Aucune logique spécifique à la fixture dans le Store/runtime. Nettoyage et restauration dans finally.

Preuves 13.5 : docs/evidence/desktop-store135-{staging,windows,artifacts}.json. Les preuves 13.2–13.4 restent historiques. Une correction préalable remplace l'ancre dry-run locale de la calculatrice par l'ancre publique du paquet effectivement publié, vérifiée cryptographiquement avant recompilation et avant le nouveau parcours.
