# Étape 16.2 — Éditeur accessible Web

Base : `2a7f367`. Date : 28/10/2026. Autorisation : propriétaire, « Enchaîne ces 3 étapes ».

Page `/workflows` déployée sur le Web staging, navigation et style du workspace conservés. Palette de recherche paginée issue de l’API publiée, actions vérifiées depuis leurs DTO réels, ajout par bouton/clavier ou glisser-déposer. Carte avec liens et boutons d’étapes, alternative liste réordonnable au clavier, inspecteur de paramètres selon le schéma et liaison de résultats typés. Aucune action inventée pour l’éditeur.

Validation immédiate : boucles, paramètres obligatoires, types/limites incompatibles, références manquantes et limites de graphe. Export désactivé sur document invalide. Import local résolvant les actions actuelles, annulation sans remplacer le document courant, annulation des modifications, export JSON téléchargé et restitution identique. Aucun stockage automatique ni envoi du document au cloud.

Le schéma gelé possède un validateur standalone généré à la compilation, sans `eval` dans le navigateur. Hydratation sous CSP avec hashes exacts ; coordonnées de carte en unités CSS explicites pour Preact 11. Nœuds distincts vérifiés après contrôle visuel. L’éditeur indique qu’il prépare le parcours, sans feindre une exécution ; raccordement en 16.3.

Validation : 31 tests Web dont six sur format/DAG/mapping/prototypes, analyse Astro (80 fichiers, zéro erreur), build, 3 contrats. Trois vrais scénarios Chromium staging réussis : deux nœuds entièrement au clavier, source typée, boucle bloquant export, sérialisation/restauration identique et téléchargement ; import annulé / action inconnue refusée ; mobile 390 px et vue liste. Zéro iframe outil et zéro POST durant le scénario de création.

Décision : **terminée et close techniquement**. Preuves : `workflows-validation.json`, `workflows-browser.json`, captures bureau/mobile et logs. Déploiement Web staging seulement ; API/runtime/quotas IA/production/DNS/messagerie conservés. 16.3 non engagée, réserve Google 14.5 inchangée ; approbation humaine non inventée.
