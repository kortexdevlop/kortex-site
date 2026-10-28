# Email et mot de passe — complément demandé avant 14.2

28/10/2026. Demande explicite : « Avant de passer à 14.2, ajoute la connexion / inscription via email mot de passe ». Le propriétaire confirme l'absence de service d'envoi d'emails. Ce complément est indépendant des clients OAuth Google encore manquants. Il ne clôture pas la validation Google et n'engage pas 14.2.

## Utilisation

- Inscription : https://kortex-web-staging.userpgl.workers.dev/auth/register
- Connexion : https://kortex-web-staging.userpgl.workers.dev/auth/login
- Compte : https://kortex-web-staging.userpgl.workers.dev/account

« Mon espace » ouvre le compte depuis le workspace ; un visiteur sans session est redirigé vers la connexion. Inscription avec confirmation du mot de passe, affichage/masquage, erreurs accessibles, compte et déconnexion. Écrans adaptés au mobile et au bureau dans les couleurs du workspace. Le lien de navigation invité reste disponible. Le bouton Google est conservé et n'apparaît que lorsque cette méthode est configurée ; Google reste désactivé sur le staging actuel.

L'email sert d'identifiant et reste **non vérifié**, ce que la page du compte indique. Aucun email envoyé, aucune récupération de mot de passe par mail disponible tant qu'un service n'est pas ajouté. Aucun rapprochement automatique avec un compte Google, même si une adresse identique y apparaît. Aucune synchronisation activée par cette inscription.

## Stockage et protection

Migration additive 0007_password_auth.sql : password_credentials avec FK utilisateur/cascade, password_auth_forms et password_auth_limits. Email ASCII normalisé par trim et minuscules ; points et suffixes + conservés. Email nécessaire à l'identification stocké en D1, index HMAC distinct avec le pepper. Aucun mot de passe en clair, log, URL ou stockage navigateur.

Scrypt natif node:crypto dans Workers, profil N=32768, r=8, p=3, sortie 32 octets, sel aléatoire de 16 octets ; pré-hachage HMAC-SHA256 avec PASSWORD_PEPPER serveur dédié. Paramètres fixes/versionnés, jamais exécutés depuis un coût choisi dans une requête ou un enregistrement corrompu. Vérification contre un vecteur indépendant Node natif. Comparaison sans sortie anticipée selon les octets. Un seul calcul à la fois par isolate, budget mémoire 40 Mio et réponse 429 plutôt qu'une file de calculs illimitée. Version/date Workers et nodejs_compat rendent cette primitive disponible ; aucun KDF maison ni dépendance cryptographique JavaScript ajoutée au produit.

Nouveaux mots de passe : 15–128 caractères Unicode, 512 octets maximum, aucun trim/normalisation du mot de passe. Quelques mots de passe très courants et répétitions triviales refusés ; ceci n'est pas une base exhaustive de mots de passe compromis. Identifiant inconnu et mauvais mot de passe : même réponse générique et même coût scrypt. Sel et secret empêchent un simple hash rapide ou une table partagée de mots de passe.

Limites persistantes D1 avant calcul : 8 connexions par identifiant/15 minutes, 3 inscriptions par identifiant/heure, 60 opérations mot de passe/minute globalement, 120 formulaires/minute et 10000 formulaires actifs maximum. Compteurs HMAC, sans journal d'adresse IP. Expiration/prune des formulaires après dix minutes et des compteurs à la fin de fenêtre. Les limites ne constituent pas une protection anti-abus exhaustive de la phase sécurité ultérieure.

## API, sessions et configuration

GET /api/v1/auth/options, GET /api/v1/auth/password/token, POST /api/v1/auth/password/register et POST /api/v1/auth/password/login. Les POST sont JSON borné à 4 Kio ; email/password seuls admis, Origin exact et preuve CSRF liée à un cookie host-only HttpOnly de dix minutes. Ces routes de cookie ne sont accessibles que via le binding privé AuthService du Web Worker, jamais sélectionnable par un header. Le relais Web transmet le corps borné et le contexte CSRF, sans bearer du navigateur.

Succès : session opaque renouvelée, ancienne session révoquée, cookie __Host-kortex_session Secure/HttpOnly/SameSite=Lax/Path=/ ; formulaire retiré. /me et logout fonctionnent sans configuration Google et réutilisent les contrôles CSRF/révocation existants. /me ajoute email/email_verified seulement pour un compte avec identifiants locaux ; aucune information de hash ou de secret n'en sort.

PASSWORD_AUTH_ENABLED=true, PASSWORD_PEPPER et SESSION_HMAC_SECRET d'au moins 32 octets sont requis. Le staging possède ces deux secrets dans Cloudflare ; ils ont été créés aléatoirement en mémoire, jamais écrits dans le dépôt ou imprimés. Aucun secret existant n'a été remplacé. PASSWORD_PEPPER ne doit pas être changé sans stratégie de migration/réinitialisation des mots de passe ; il protège aussi l'index d'identifiants. Google garde ses propres variables OAuth. Le local et la production n'activent pas automatiquement cette méthode ; aucune ressource production, DNS ou messagerie modifiée.

## Vérification

9 nouveaux scénarios backend, 47 tests API de régression, 14 tests proxy, 3 contrats, types API et analyse Astro de 60 fichiers sans erreur. Build Web et contrôles de sortie SSR réussis. Parcours réel Chromium sur staging : inscription, email non vérifié, cookie Secure/HttpOnly/host-only, rechargement, refus logout sans CSRF, déconnexion, refus mauvais mot de passe puis connexion correcte. Affichage 1360px et 390px vérifié ; compte de test supprimé avec ses identifiants/sessions par cascade. Les métadonnées temporaires de formulaires/limites expirent normalement. Aucun email envoyé. Preuves : docs/evidence/password-auth-validation.json et password-auth-staging.json.

Sources : [OWASP Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html), [Cloudflare node:crypto](https://developers.cloudflare.com/workers/runtime-apis/nodejs/crypto/).
