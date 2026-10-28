# Étape 15.1 — Propositions Workers AI strictes

Base : `b00a6af`. Autorisation : propriétaire, « Enchaines ces 3 étapes ». Date : 28/10/2026.

`POST /api/v1/ai/plan` est actif sur le staging via la façade privée Web. Session propriétaire, CSRF et consentement obligatoires. Le jeton Desktop n’a aucun droit IA. Message/IDs bornés, champs supplémentaires et fichiers rejetés avant le fournisseur. Catalogue de vingt actions maximum, exclusivement publiées/vérifiées et non révoquées.

Workers AI reçoit une grammaire incluant les schémas exacts des actions candidates. Le serveur valide à nouveau structure, version, consentement, action et paramètres via l’interpréteur du sous-ensemble JSON Schema V1 sans compilation dynamique. Taille de plan 32 Kio, dix étapes, timeout et quotas appliqués. Aucune invocation ou exécution de module dans le Worker.

Les deux premiers appels ont été refusés avec statut 503 ; le diagnostic du second a identifié des paramètres invalides. La grammaire fournisseur initiale laissait `input` libre ; elle a été remplacée par les schémas des actions candidates. L’essai suivant réel a renvoyé `calculatrice/add`, `{a:2,b:3}`, `requires_consent:true`, statut 200. Il n’a pas exécuté le calcul. Les erreurs ne révèlent aucun contenu modèle ni exception fournisseur libre.

Validation : 81 tests API réussis, dont réponse fournisseur mal formée, action inconnue, paramètres faux, schéma fournisseur contextualisé, redaction, privé/CSRF/invité et réservations concurrentes. Types, Web/contrats réussis ; vrai binding Cloudflare, vraie session browser, refus privé 400, proposition 200 et quota 429. Migration 0009, API et Web staging déployés ; secrets existants conservés. Preuves datées et hashes dans `device-14.5-15.1-validation.json`.

Décision : **réalisation et validation techniques 15.1 terminées**, réserve OIDC 14.5 explicitée au registre. Aucun chat produit, médiateur local ou exécution IA livré avant 15.2. Production, DNS et messagerie non touchés.
