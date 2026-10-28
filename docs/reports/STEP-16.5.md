# Étape 16.5 — Orchestration entre outils

**Terminée et close techniquement le 28/10/2026.** Base du complément : `5c9a78b`. La réserve des autorisations Windows de 16.3 est levée ; aucune approbation humaine formelle n’est inventée.

Les scénarios réels des paquets signés restent vérifiés : date UTC → calculatrice, 1200 puis 1210, sur Web et Windows ; calculatrice `number` → timestamp `integer` refusé sans coercition ; action de statistiques texte absente refusée ; fichier `abc` → SHA-256 identique, lecture explicitement consentie et aucune donnée de fichier dans la définition sauvegardée. Le sens positif demandé initialement est remplacé par le sens date → calcul compatible avec les contrats gelés ; le sens initial demeure un test négatif.

Le complément exécute un même paquet SDK signé sensible dans le moteur natif et dans le Worker WASM du navigateur, par les adaptateurs produits et les courtiers existants. L’écriture autorisée puis la lecture donnent `private-test-value` dans les deux backends réels. Le refus n’écrit rien. Windows vérifie en plus qu’un scope détourné ne produit aucun effet, qu’un droit à usage unique ne permet pas une seconde écriture, que l’annulation conserve seulement les étapes déjà achevées et qu’un autre outil ne lit pas le même namespace. Le navigateur attend le consentement avant de terminer l’étape et refuse le rejeu d’un identifiant de requête.

L’essai Windows dans l’interface réelle vérifie refus, accord écriture/lecture, détournement refusé et annulation à la fermeture de la vue. Les six scénarios du Web staging couvrent aussi la réutilisation d’une seule instance pour deux actions, la fermeture avant accord, les types/actions invalides, le fichier lu zéro fois avant accord puis une fois après, le téléchargement local explicite et la restauration d’un brouillon incomplet sans invocation.

Les outils/IDs et le code adversarial vivent uniquement dans les fixtures de tests. Aucun outil de test n’est publié sur le catalogue staging. Les paquets métier n’ont pas été changés ; l’ordonnanceur n’a aucune branche par ID d’outil. Les limites et refus du profil restent ceux du contrat et des courtiers hôtes, notamment l’absence de stockage Web durable sans accord du navigateur.

Preuves : `workflow-permissions-validation.json`, `workflow-permissions-web.json`, `workflow-permissions-ui.json`, preuve native sensible, régressions, et preuves antérieures des trois scénarios métier. Windows/Chromium uniquement, selon les exclusions demandées. Aucune interface 17.1 engagée.
