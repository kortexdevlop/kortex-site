# Clés, rotation et révocation

Référence opérationnelle : [runbook de rotation](runbooks/rotate-keys.md). Les formats et leurs gels restent décrits dans [CONTRACTS](CONTRACTS.md) et [SIGNING](SIGNING.md). Aucun secret privé n'est inclus dans ces documents.

Une rotation normale conserve une chaîne de confiance vérifiable et des compteurs croissants ; le remplacement d'une racine compromise demande un canal indépendant et une migration des hôtes. Les racines hors ligne, clés déléguées, jetons Cloudflare, identité OIDC de publication et sessions Google/Web/Windows ne sont pas interchangeables.

Les clés de test éphémères ne sont pas des identités de production. Une restauration de données ne peut pas effacer un retrait ou abaisser le minimum accepté par un client. Tester archives altérées, signature invalide, rejeu, expiration, retrait et reconnexion avant de reprendre une publication.

En incident : suspendre les mutations concernées, appliquer le [retrait d'outil](runbooks/revoke-tool.md) si nécessaire, conserver les preuves expurgées et suivre [RECOVERY](RECOVERY.md). Aucun secret n'est demandé dans un ticket public ou une commande documentée en clair.
