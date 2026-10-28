# Étape 16.4 — Persistance et synchronisation facultative

28/10/2026. Base : `16de346`. Implémentation et tests techniques terminés ; approbation humaine non inventée.

Sauvegarde explicite des définitions sur l’appareil, sans compte ; restauration des brouillons et revérification des actions actuelles. Le Web garde au plus 20 définitions et 4 Mio ; Windows utilise une écriture locale atomique contrôlant les liens de fichiers. Les fichiers sélectionnés et résultats ne modifient pas la définition. Les valeurs de fichiers, handles et chemins sont refusés à la sauvegarde ; une valeur vide `data_base64` peut servir d’emplacement pour une prochaine sélection locale. Cette protection ne prétend pas reconnaître toute donnée privée saisie manuellement dans un texte libre.

Migration additive **0011**, sans réutiliser le numéro 0005 déjà appliqué. Table `user_workflows`, propriétaire, révision globale monotone, révision par ligne et tombstones. `GET/POST /me/workflows`, `GET/PUT/DELETE /me/workflows/:id` ; les écritures demandent `If-Match: "workflows-N"`. Synchronisation du compte obligatoirement activée, scopes natifs existants, CSRF Web, façade Web cookies uniquement, Origin refusé pour les tokens natifs. Pas de sélection d’un autre utilisateur par query ou en-tête. Quotas : 100 lignes avec tombstones, 256 Kio par requête et 512 Kio de définitions cumulées.

Les conflits 409 préservent la copie locale et imposent une recharge/comparaison explicite. La révision des parcours entre dans l’ETag opaque d’export/purge du compte (`account-L-P-W`) pour empêcher une suppression à partir d’un export antérieur à une modification de parcours ; les anciens clients doivent continuer de transmettre l’ETag reçu sans le construire. Les parcours font partie de l’export et sont supprimés en cascade avec leur propriétaire.

Windows utilise le coffre et le transport de compte existants. Le propriétaire capturé à la lecture est revérifié à l’écriture ; aucun token n’est exporté. L’interface permet sauvegarde locale, lecture de la liste et sauvegarde cloud explicite.

Tests API de propriétaire/opt-in/CAS/tombstones/purge/scopes natifs/Origin/révocation ; scénario réel D1 staging avec compte jetable supprimé après vérification ; tests locaux Windows sur vraie écriture/relecture et rejet de données privées. Les validations des permissions sensibles de 16.3 restent indépendamment ouvertes.
