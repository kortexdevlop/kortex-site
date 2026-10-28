# Rapport 14.0 — Google et comptes facultatifs

28/10/2026. Base ad00aa5. Autorisation : « Enchaines ces 2 étapes rapidement ». Fournisseur choisi explicitement : Google. Le propriétaire confirme que les clients OAuth ne sont pas encore créés.

Schéma D1 et configuration préparés : users, external_identities, sessions HMAC-hachées, événements minimaux et expiration. Migration additive 0005 : 0003/0004 déjà utilisées. Discovery bornée, HTTPS et origines explicitement autorisées, S256, JWKS publics ; aucune route de connexion ou validation JWT improvisée. Google désactivé tant que les identifiants clients manquent.

Tests SQL/configuration dans api/tests/auth-schema.test.ts ; suite API complète, types et migrations locales : docs/evidence/identity-14.0-validation.json. Discovery et JWKS Google interrogés réellement et S256 vérifié. Les transports contrôlés des tests locaux sont identifiés, sans prétendre à une connexion réussie.

Préparation locale réalisée ; clôture et validation staging en attente des clients OAuth Web/Desktop, écran de consentement et comptes de test. Aucune connexion réelle testée ; aucun compte créé. Guide : docs/AUTH.md. Comportement anonyme inchangé. 14.1 non engagée ; production, DNS et messagerie intacts.
