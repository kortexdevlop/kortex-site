# Étape 15.3 — Interface assistant Web/Desktop

Base : `50159f2`. Date : 28/10/2026. Autorisation : propriétaire, « Enchaîne ces 3 étapes ».

Page `/ai` et onglet Windows « Assistant » reliés à l’API et aux moteurs locaux. Demande avec consentement cloud, propositions vérifiables, approuver/refuser par action, résultat local seulement après réponse validée. Pas d’exécution à réception. Erreurs explicites : quota, hors connexion, droit IA absent, schéma refusé, échec local. Plans/résultats en mémoire ; déconnexion/changement de compte et expiration contrôlés avant exécution.

Le Web charge ses scripts d’hydratation sous CSP via hashes exacts du rendu serveur, sans `unsafe-inline` de script. Saisie désactivée avant hydratation/catalogue. Le runtime reste sur une origine dédiée et le consentement sensible reste dans l’outil. Le hash de fichier est un parcours local explicite distinct de l’IA (16 Ko maximum pour cet exemple).

Le droit IA natif reste désactivé pour chaque appareil. Migration 0010 ; activation/retrait uniquement depuis les réglages Web propriétaires sous CSRF. Un appareil ne peut pas l’accorder lui-même. Les scopes `/me` reflètent ce consentement. Compte et quotas partagés Web/Desktop. Vrai parcours : connexion navigateur/coffre, refus IA par défaut, activation Web, quota 429 réel, retrait immédiatement effectif, purge du compte temporaire et coffre nettoyé.

Validation : 82 tests API complets, puis 21 tests auth/appareil ciblés après l’ajustement final des scopes ; 25 tests Web, 3 contrats, types API/adaptateur et analyse Astro (73 fichiers, zéro erreur), builds réussis. Quatre scénarios Chromium staging et interface Windows réelle : refus puis 25×48=1200 hors connexion. Propositions fixes exclusivement injectées par les tests ; aucun bypass produit. Génération réelle Workers AI déjà prouvée en 15.1, quota non relevé.

Chemins : pages/composant/lib/middleware/navigation Web, UI et compte Desktop, permission appareil API/proxy/SQL, tests/scripts staging. Production, DNS et messagerie conservés.

Décision : **terminée et close techniquement**. Captures et preuves : `assistant-web.png`, `assistant-windows.png`, `assistant-account-staging.json`, `assistant-native.json`, agrégat `assistant-validation.json`. Revue humaine non inventée ; Google reste en attente de ses clients OAuth.
