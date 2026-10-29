# Utiliser Kortex

Kortex exécute les outils sur votre appareil : dans un cadre isolé du navigateur ou dans l’application Windows. Le catalogue, les téléchargements et les fonctions de compte utilisent le réseau. Le traitement local n’est pas une promesse que toute fonction de la plateforme est hors connexion.

## Accéder au service

La version disponible est le [site de staging](https://kortex-web-staging.userpgl.workers.dev/). Il s’agit d’un environnement de validation ; la production sur le domaine définitif n’est pas encore livrée. Utiliser Chromium sur Windows pour le périmètre testé.

La [préversion Windows 1.0.0](https://github.com/kortexdevlop/kortex-site/releases/tag/desktop-staging-v1.0.0) fournit un installateur EXE x64, son checksum et sa provenance. Télécharger depuis cette release, comparer le SHA-256 publié, puis lancer l’installateur si l’on accepte cette préversion. WebView2 x64 est inclus. L’application Kortex elle-même n’est pas encore signée Authenticode : la signature Microsoft du composant WebView2 et la provenance de release ne remplacent pas cette signature. La distribution de production attend un certificat Windows réel ; ce guide ne demande pas de désactiver SmartScreen ou l’antivirus.

Les essais Linux/macOS et autres navigateurs restent exclus. Ne pas considérer un fichier d’une autre plateforme comme un installateur validé.

## Choisir un outil et travailler dans le bureau

Depuis l’accueil, rechercher un outil ou ouvrir le catalogue. Une fiche décrit ses actions et permissions. Dans **Bureau**, choisir un outil puis **Ouvrir une fenêtre**. Jusqu’à huit fenêtres peuvent coexister ; chacune garde son instance et ses autorisations. Réduire, agrandir, restaurer et fermer avec les commandes du titre. Sur une petite largeur, choisir l’onglet de l’outil.

Pour les commandes au clavier : Tab et Maj+Tab parcourent les contrôles ; Entrée/Espace activent les boutons ; le lien **Aller au contenu** évite la navigation ; Échap ferme les dialogues. La recherche globale utilise Ctrl+K sous Windows. Sur le titre d’une fenêtre, Alt+flèches la déplace et Alt+Maj+flèches la redimensionne.

Les outils déjà chargés peuvent continuer à calculer après une coupure réseau. L’ouverture d’une nouvelle fenêtre ou un nouveau téléchargement exige la connexion. Un outil installé sur Windows reste soumis à la validité de sa confiance et de ses informations de révocation : un besoin de reconnexion n’est pas une autorisation de contourner la vérification.

## Fichiers et permissions

Choisir volontairement un fichier depuis un outil qui le demande. Lire le dialogue de consentement, sa ressource et sa durée, puis autoriser ou refuser. Un refus doit laisser l’outil utilisable sans afficher un ancien résultat comme s’il était nouveau. Un droit à usage unique n’autorise pas automatiquement l’accès suivant.

Les fichiers choisis ne sont pas automatiquement envoyés à un serveur ou à l’IA. Les handles et droits sont temporaires ; fermer l’instance les révoque. Un outil autorisé à utiliser son stockage local peut conserver les données qu’il y écrit : fermer une fenêtre ne purge pas ce stockage. La disposition et l’historique du bureau ne doivent pas contenir les fichiers, entrées ou résultats.

Pour un export, autoriser l’enregistrement proposé et choisir sa destination. Une autorisation d’ouvrir un fichier ne vaut pas autorisation de sauvegarder un autre fichier. Le détail des limites et de la suppression est dans [PRIVACY](PRIVACY.md).

## Compte, synchronisation et Windows

Les outils peuvent être utilisés sans compte. Pour créer un compte : **Mon espace**, puis **Créer un compte**, ou `/auth/register`. La connexion email/mot de passe est disponible ; une phrase de 15 à 128 caractères convient si elle respecte les contrôles affichés. L’adresse reste non vérifiée et la récupération du mot de passe par email n’est pas encore disponible, faute de service d’envoi.

Google a été choisi comme fournisseur, mais ses clients OAuth ne sont pas configurés : ne pas considérer le bouton Google comme disponible. L’absence de Google ne désactive pas la connexion email.

La synchronisation des favoris, réglages et définitions de parcours est facultative. L’activer depuis le compte ; elle n’envoie pas automatiquement les fichiers ou résultats d’outils. Une définition de parcours contenant une constante textuelle peut néanmoins contenir une information personnelle : examiner ses paramètres avant de la synchroniser. Les conflits et suppressions sont gérés par révision, pas par écrasement aveugle.

Pour connecter Windows, lancer la connexion de compte dans l’application puis terminer la confirmation dans le navigateur. Le jeton d’appareil est conservé dans le coffre Windows. L’autorisation IA d’un appareil est distincte et révocable depuis le compte. Déconnexion, révocation d’appareil et suppression du compte n’ont pas la même portée ; examiner la confirmation avant une suppression.

## Assistant et parcours

L’assistant reçoit uniquement le message que l’on accepte de transmettre. Ne pas y mettre de fichier privé, clé ou secret. Il propose des actions ; l’exécution reste locale après examen et consentement. Une limite quotidienne Kortex peut être atteinte alors que le quota Cloudflare est encore disponible : les messages les distinguent.

Dans **Parcours**, choisir des actions publiées, relier leurs entrées/sorties et vérifier les paramètres. Préparer puis confirmer l’exécution. Un consentement à un parcours ne remplace pas les permissions sensibles d’une étape. Utiliser **Arrêter l’exécution** pour empêcher les étapes suivantes ; cela n’annule pas nécessairement un effet déjà terminé, comme un fichier déjà sauvegardé.

Les parcours s’exécutent dans une session ouverte ; aucun traitement cloud en arrière-plan, GPU guest ou formule payante n’est annoncé par cette version.

## En cas de problème

| Message ou situation | Suite utile |
|---|---|
| Catalogue indisponible | Garder les fenêtres chargées, réessayer après rétablissement ; une panne D1 du compte peut toucher staging |
| Signature, intégrité ou révocation refusée | Reconnecter et vérifier la version ; ne pas ouvrir un paquet modifié ou désactiver le contrôle |
| IA indisponible ou limite atteinte | Ouvrir les outils manuellement ; aucun outil n’a été exécuté du seul fait de cette erreur |
| Permission refusée | Choisir à nouveau l’accès si nécessaire ; vérifier le fichier et la ressource du dialogue |
| Stockage local indisponible | Vérifier le mode privé et les autorisations du navigateur ; un export explicite reste distinct |
| Mise à jour Windows interrompue | Conserver l’ancienne version, relancer une fois le service rétabli ; ne pas modifier les archives installées |

Pour signaler un défaut, utiliser les [issues du dépôt](https://github.com/kortexdevlop/kortex-site/issues) avec version, plateforme, étapes et message affiché. Retirer fichiers privés, email, cookies et jetons du signalement. Ce guide ne promet pas une astreinte déjà opérationnelle ; les alertes continues attendent encore leur activation.
