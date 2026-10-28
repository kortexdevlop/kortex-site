# Rapport 14.1 — Connexion Web et sessions

28/10/2026. Base 53a0c02. Autorisation explicite : « Enchaine la prochaine étape rapidement », puis « Termine », malgré l'attente externe des identifiants OAuth de 14.0.

Endpoints login, callback, logout et me implémentés. Code exchange + PKCE S256 ; state lié au navigateur, nonce et configuration, consommation atomique anti-rejeu et vérificateur chiffré. Validation RSA/JWKS/issuer/audience/azp/exp/iat/nbf. Sessions opaques HMAC, cookies host-only Secure/HttpOnly/SameSite=Lax ; rotation et révocation. Protection Origin/CSRF pour logout. Google ID token Desktop limité à identité connue et lecture seule, sans autorisation d'écriture déduite.

Relais Astro /api/v1 et alias /auth/callback, binding privé AuthService ; aucun accès par header à l'entrypoint privé. Aucune session iframe, aucun bearer navigateur transmis. Redirections et cookies filtrés, erreurs génériques, no-store/no-referrer. Le secret Google reste dans l'API Worker ; aucun secret Web/public. Migration additive 0006 pour les transactions de dix minutes. Aucun ancien contrat supprimé ni renommé.

47 tests API, 13 tests ciblés proxy et trois tests contrats réussis ; types API, analyse des 54 fichiers Astro sans erreur et builds locaux API/Web réussis. Les tests utilisent D1 émulée et RSA réels avec un transport IdP contrôlé, sans bypass de vérification. Preuves et hashes : docs/evidence/identity-14.1-validation.json.

Implémentation locale terminée. Clôture staging en attente : clients Google Web/Desktop, consentement/comptes de test et secrets serveur, puis connexion réelle navigateur et vérification des cookies/logout. Aucun login Google réel revendiqué et aucun déploiement dans ce lot. Production, DNS et mail inchangés. Guide docs/AUTH.md ; 14.2 non engagée. Statuts formels historiques conservés, sans approbation visuelle inventée.
