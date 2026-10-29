# Retirer un outil ou une version

Identifier outil, version et SHA exacts depuis les reçus et le catalogue, sans ouvrir un paquet suspect dans le profil d'un utilisateur. Suspendre les publications concurrentes. Si la clé est suspecte, suivre aussi [rotate-keys](rotate-keys.md).

Préparer un document de révocation signé hors CI avec compteur supérieur, dates valides et tous les tombstones existants. Le retrait doit être connu des clients avant de considérer le catalogue comme sécurisé. Ne pas réduire un compteur ou effacer un retrait pour réussir un rollback.

Commiter un plan de maintenance relu sous `infra/releases/` : `action: revoke`, `request_id` stable, outil/version/SHA, motif et document de révocation. Le déclenchement protégé `tools.yml` avec `operation=revoke` vérifie la CI et l'identité. Les champs réels sont décrits dans [infra/releases](../../infra/releases/README.md).

Rejouer le même plan après une coupure, pas une nouvelle requête contradictoire. Vérifier réponse 410 au téléchargement, admission refusée avant création de Worker/guest, et conservation de la trace d'audit. Un ancien ETag, une archive installée ou la remise d'un ancien pointeur D1 ne doivent pas contourner le retrait. Un calcul déjà commencé hors ligne ne peut pas être présenté comme stoppé à distance.

Conserver les octets pour investigation selon les droits/rétentions définis ; le retrait du catalogue n'est pas une purge physique de toutes les copies. Pour un outil corrigé, publier une nouvelle version autorisée et vérifier les cinq parcours de référence sans réhabiliter le paquet compromis.
