# Rapport 11.2 — Accueil réel selon la nouvelle référence

27/10/2026, Windows. Base aa59e70. Commit résultat dans PROGRESS.md. Préparation implémentée et testée sur instruction de poursuivre le lot ; statut formel pending derrière la revue visuelle existante.

La demande utilisateur du 27/10 remplace le hero deux colonnes et les neuf grandes cartes pastel du plan initial par Workspace-selection.png : sidebar, titre centré, demande/fichier, suggestions, récents et cartes blanches en quatre colonnes. Réutilisation de Workspace/WorkspaceLayout, sans créer un deuxième Hero ou FeaturedTools contradictoire. Aucune fausse activité récente, connexion Desktop ou carte du raster dans le rendu public.

L’accueil charge au plus 24 vrais résumés depuis l’API, avec accès au catalogue paginé. Envoi de la demande, recherche globale et suggestions naviguent vers une requête API ; catégories publiques issues des données réelles. Les cartes sont des liens SSR vers la fiche et restent utilisables sans JavaScript. Les démonstrations sont confinées à l’atelier de développement, absent du build. Le dépôt reste local et n’annonce aucun traitement inexistant.

Tests sur Worker compilé : exactement cinq entrées D1 trouvées par recherche donnent cinq cartes réelles, pas neuf imposées. Recherche Calculatrice depuis le formulaire, ouverture de la fiche, absence de récents factices et de Desktop connecté. Absence d’overflow à 2706, 1353, 768, 390 et 320 pixels. Captures home-real-chromium.png et home-real-webkit.png.

Régression séparée de l’atelier : géométrie native ±4 px, référence SHA inchangée, clavier, filtres/modes/favoris, fichier local et menu mobile. Les nouvelles preuves sont écrites dans docs/evidence/website-workspace-regression pour préserver les empreintes du lot précédent. Les faibles contrastes de la référence restent documentés, sans conformité AA globale revendiquée.

Build et typecheck réussis ; suite intégrée Chromium/WebKit et régression visuelle référencées dans docs/evidence/website-11.1-11.3-validation.json. Aucun déploiement ou changement du domaine/messagerie.
