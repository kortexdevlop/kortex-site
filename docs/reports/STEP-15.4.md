# Étape 15.4 — Cinq outils et confidentialité

Base : `50159f2`. Date : 28/10/2026. Autorisation : propriétaire, « Enchaîne ces 3 étapes ».

Les cinq outils sont exécutés réellement via les médiateurs : multiplication 1200, UTC/Unix 1704067200, casse Unicode, CSV deux lignes, SHA-256. Catalogue/admission/modules Web staging réels ; Wasmtime Windows et paquets signés réels pour les cinq actions, sans réseau. Les propositions déterministes sont confinées aux tests, sans générateur artificiel dans l’application.

Le cas privé utilise un vrai fichier sélectionné localement et une approbation distincte. Trace complète des requêtes : aucun octet sentinelle, base64 ou nom du fichier transmis au réseau, aucun appel IA et aucune persistance du contenu dans localStorage. Le module reçoit les octets explicitement choisis pour le contrat pur `sha256_bytes` ; aucune permission `file.read` n’est créée par le plan. Résultat local non envoyé au cloud.

Refus, plan mal formé, action « delete-files » inconnue, paramètres faux, expiration, rejeu et déconnexion refusés sans invocation. L’arrivée de plan ne crée aucun iframe. Quota/droit IA réels vérifiés sur le staging ; hors connexion, calculateur natif et outils manuels restent disponibles. Marqueur fichier privé natif conservé.

Validation : quatre tests Chromium staging réussis, un test adaptateur signé, 21 cœur, 25 Desktop distincts (suite complète 24 puis matrice ajoutée), 82 API et 21 auth ciblés, 25 Web, 3 contrats, types/analyse/build. Les générations réelles des cinq plans n’ont pas été relancées après épuisement des trois réservations quotidiennes. Aucune remise à zéro ni hausse de quota ; la preuve réelle d’inférence du lot 15.1 est référencée. L’assistant natif n’exécute pas les actions sensibles : retour vers l’outil et son consentement normal, limite documentée.

Décision : **terminée et close techniquement dans le périmètre Windows/Chromium demandé**. Matrice : `docs/evidence/ai-action-matrix.md`, traces `assistant-privacy.json`, `assistant-four-tools.json`, `assistant-native-matrix.json`, agrégat `assistant-validation.json`. Pas de Linux/macOS/Firefox ni de validation Google inventée. Production, DNS et messagerie préservés. Phase 16 non engagée.
