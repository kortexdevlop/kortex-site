# Rapport 11.5 — Recherche et accessibilité

27/10/2026, Windows. Base 524a6b0. Implémentation autorisée dans le lot 11.4–11.6. Commit résultat dans PROGRESS.md ; statut formel pending derrière la revue visuelle existante.

Une seule île Preact SearchDialog dans Workspace, sans réintroduire l’ancien Header horizontal ou modifier le dessin au repos. Recherche API après 250 ms de saisie, q borné à 100 caractères, annulation AbortController et génération de requête pour ignorer une ancienne réponse même si son transport ignore l’annulation. Chargement, vide et erreur annoncés ; liens vers les fiches et formulaire de résultats complets. Catégorie/tri restent dans l’URL du catalogue.

Ctrl/⌘K ou clic sur la recherche ouvre un dialogue natif avec focus, fermeture Escape, résultats accessibles au clavier et ArrowDown. Les préférences FR/EN du chrome et Aa sont placées dans le dialogue pour respecter la maquette au repos, puis mémorisées localement. Le contenu des outils n’est pas prétendu traduit. Depuis /app, le raccourci ouvre la recherche du catalogue : aucune île nécessitant un bootstrap inline n’est injectée dans le shell d’exécution à CSP stricte. Les sessions Astro inutilisées sont désactivées ; aucune KV distante n’est créée pour une fonction qui n’est pas utilisée.

Essais Chromium/WebKit : raccourci/focus/fermeture/réouverture, recherche réelle, vide, panne, debounce, réponse obsolète, langue/texte mémorisés et zéro violation axe critical dans le dialogue. Une course au démarrage de l’hydratation WebKit a été corrigée avec installation synchrone des écouteurs, signal de disponibilité et reprise du focus initial ; les huit essais locaux finaux passent.

Performance Cloudflare réelle : D1 et Worker temporaires, 10 000 entrées synthétiques isolées du catalogue du produit, 25 aller-retours HTTPS incluant lecture des résultats, p95 **430,69 ms**, cible API <500 ms atteinte. Le délai UI de 250 ms n’est pas inclus dans ce p95 réseau/API et n’est pas présenté comme une mesure complète de l’interaction utilisateur. Worker et D1 de benchmark supprimés à la fin (cleanup=true). Aucune fixture permanente dans le produit. Rapport : docs/evidence/search-staging-benchmark.json.

Les faibles contrastes du raster de référence restent une réserve distincte : aucune conformité WCAG AA globale revendiquée. Linux/macOS et Firefox exclus par instruction du propriétaire.
