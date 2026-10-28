# Étape 17.2 — Actions et parcours dans les fenêtres

28/10/2026, base 10da685. Terminée et close techniquement sur Windows/Chromium.

WorkspaceActions résout une instance possédée par le contrôleur, son outil, sa version et ses actions publiées. Entrées et sorties validées, borne 1 Mio, une invocation à la fois par fenêtre, nouvelle vérification du contrat en ligne, métadonnées déjà chargées en mode hors connexion. Le canal exige source/origine/nonce/session/requête concordants ; aucun message guest ne peut demander au routeur de cibler une autre fenêtre.

Le panneau propose une action et une fenêtre existante. Le dernier résultat s’affiche dans cette fenêtre et dans le panneau local, uniquement en mémoire. Un parcours sauvegardé/importé est validé puis chaque nœud est lié explicitement à une fenêtre existante ; le moteur DAG et les courtiers existants exécutent réellement. Consentement de parcours distinct des permissions sensibles. Arrêt explicite, fermeture pendant une requête et fermeture d’une fenêtre future interrompent la suite ; résultats partiels conservés seulement en mémoire.

Historique local borné à cent lignes : date, outil, action, état uniquement. Aucun paramètre, résultat, fichier, nom privé ou handle ; aucune synchronisation du contenu du bureau. Fermer invalide immédiatement la génération et annule les requêtes. Une identité retirée ne peut être réattachée dans ce contrôleur.

Validation réelle : calcul 1200 dans une instance, 5 dans l’autre et 7 hors connexion ; résultats présents seulement dans la bonne fenêtre. Parcours interrompu après fermeture de sa prochaine fenêtre ; arrêt avant consentement sans action exécutée. Routeur de production bundlé pour le test, vrais moteurs signés : mauvais outil FORBIDDEN, identité inconnue/fermée NOT_FOUND, réutilisation DUPLICATE, commande guest falsifiée ignorée, fermeture durant la lecture du contrat annulée. Aucun exécuteur simulé ou module de test publié. Voir desktop-playwright.log et desktop-router.json.

Les fichiers de deux outils et les droits expirés sont contrôlés en 17.3. Les restrictions normales des courtiers restent applicables. Aucun contrat guest modifié ; pas d’IA ou d’exécution cloud ajoutée. Clôture technique distincte de la revue humaine.
