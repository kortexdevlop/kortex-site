# Identité facultative — étape 14.0

Google choisi par le propriétaire le 28/10/2026. Configuration publique : `infra/identity/google-staging.json`. Connexion désactivée tant que les clients réels ne sont pas fournis et que 14.1 n'implémente pas les flux. Aucun compte réel créé ; aucun jeton accepté par le code de cette étape. Le catalogue et l'application anonyme restent accessibles. Aucune dépendance de connexion pour les outils locaux.

## Configuration et reprise

Dans Google Cloud Console > Google Auth Platform, choisir le projet Kortex, renseigner Branding et Audience, ajouter les comptes de test si l'application est en mode test. Créer deux clients séparés :

1. Application Web : URI de redirection exacte `https://kortex-web-staging.userpgl.workers.dev/auth/callback`. Origine Web : `https://kortex-web-staging.userpgl.workers.dev`. Le secret appartient uniquement au serveur/BFF Web ; jamais à un fichier suivi ou au navigateur.
2. Application de bureau : client Desktop, redirection loopback `http://127.0.0.1:49152/callback`. Le port devra être réservé et vérifié par le client natif. Navigateur système, jamais connexion Google dans WebView2. Le client Desktop est public et n'utilise aucun secret comme preuve d'identité.

Renseigner uniquement les IDs publics dans la configuration. Déposer le secret Web via le gestionnaire de secrets du serveur, pas dans le chat. Staging et production utilisent des clients distincts ; aucun client production créé ici. Vérification publique : `node --experimental-strip-types scripts/check-google-identity.mjs`. Ce contrôle prouve discovery/JWKS/S256, pas l'enregistrement des clients ni une connexion réussie.

## Flux prévus en 14.1

Authorization Code + PKCE S256 ; `state`, `nonce` et vérificateur aléatoires à usage unique, durée maximale 10 minutes ; URL de retour exacte. Le flux Web termine dans le BFF de l'origine Web et remet un cookie `__Host-kortex_session`, Secure, HttpOnly, SameSite=Lax, Path=/, sans Domain. Aucune session ou jeton dans localStorage, les URL ou les logs. Les requêtes de modification devront vérifier origine et anti-CSRF ; aucun CORS générique authentifié. Le Desktop utilise le navigateur système et un listener loopback borné et temporaire. Scopes initiaux : `openid` seulement ; pas de Gmail/Drive ni de rapprochement automatique par email.

Avant toute session, le futur vérificateur devra contrôler signature sur clés publiques JWKS connues, algorithme autorisé, issuer, audience du bon client, azp si nécessaire, expiration, nonce et usage unique. Refuser JWT forgé, `none`, token expiré, mauvaise audience, substitution de clé et rejeu. La découverte implémentée ici impose HTTPS hors sandbox locale, issuer exact, origines des endpoints explicitement autorisées, redirections HTTP refusées, délai de 10 secondes et réponses de 64 Kio maximum. Aucun endpoint ne provient d'un jeton utilisateur.

## Données, sessions et conservation

Migration additive **0005_auth.sql** : 0003 et 0004 sont déjà déployées pour d'autres fonctions, leur numérotation historique est conservée. Tables users, external_identities, sessions et auth_events. UUID interne ; unicité `(issuer,subject)` ; suppression utilisateur en cascade ; synchronisation désactivée par défaut. Ni email ni nom exigés.

Session future : 32 octets aléatoires encodés base64url ; D1 ne conserve qu'un HMAC-SHA256 avec secret serveur dédié d'au moins 32 octets. La session active doit être non révoquée et non expirée. Expiration maximale sept jours. Les événements ne contiennent que type, ID opaque et dates, sans IP, email, subject, jeton ou texte d'erreur sensible. TTL maximum sept jours. `expireIdentityData` purge sessions expirées/révoquées et événements expirés ; son déclenchement récurrent et les routes logout/export/delete viennent aux étapes suivantes. Aucune prétention de purge planifiée déjà déployée.

## Preuves et limites

Huit tests Cloudflare locaux sur la vraie base D1 émulée : migrations, unicité/FK, session unique, expiration/révocation/TTL/cascade, HMAC et entrées JWT refusées comme tokens de session ; discovery sandbox par transport contrôlé, substitution issuer, absence S256, endpoint interdit, réponse trop volumineuse. La découverte et les JWKS Google sont vérifiés séparément sur le service réel. Pas de test de connexion réel tant que les clients OAuth manquent. L'étape 14.0 reste en attente de ce provisionnement externe ; 14.1 non engagée.

Sources : [Google OIDC](https://developers.google.com/identity/openid-connect/openid-connect), [Google Desktop OAuth](https://developers.google.com/identity/protocols/oauth2/native-app), [OIDC Discovery](https://openid.net/specs/openid-connect-discovery-1_0.html), [PKCE RFC 7636](https://www.rfc-editor.org/rfc/rfc7636).
